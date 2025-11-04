# Mining Pool Implementation Plan

## Based on ORE SDK Research

This plan is based on thorough research of the ORE v3 SDK and actual capabilities.

---

## Phase 1: Smart Contract Foundation

### 1.1 Update Cargo Dependencies

**File:** `mining-pools/programs/mining-pool/Cargo.toml`

```toml
[dependencies]
anchor-lang = "0.29.0"
anchor-spl = "0.29.0"
ore-api = { git = "https://github.com/regolith-labs/ore", branch = "master" }
entropy-api = { git = "https://github.com/regolith-labs/entropy", branch = "master" }
solana-program = "1.17"
```

**Why:** Need `entropy-api` for entropy account in deploy CPI calls.

### 1.2 Define Account Structures

**Pool State:**
```rust
#[account]
pub struct Pool {
    pub authority: Pubkey,           // Pool authority PDA
    pub authority_bump: u8,          // PDA bump seed
    pub strategy: PoolStrategy,      // 25-square or 18-square
    pub total_shares: u64,           // Total shares issued
    pub total_sol_deposited: u64,    // Total SOL deposited by users
    pub total_sol_current: u64,      // Current SOL balance
    pub total_ore_claimed: u64,      // Total ORE claimed from mining
    pub last_round_id: u64,          // Last round mined
    pub paused: bool,                // Emergency pause
    pub fee_collector: Pubkey,       // Where management fees go
    pub fee_basis_points: u16,       // Fee in basis points (200 = 2%)
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy)]
pub enum PoolStrategy {
    TwentyFiveSquare,  // Cover all 25 squares (100% win rate)
    EighteenSquare,    // Cover 18 least crowded (72%+ win rate)
}
```

**User Deposit:**
```rust
#[account]
pub struct UserDeposit {
    pub user: Pubkey,               // User's wallet
    pub pool: Pubkey,               // Which pool
    pub shares: u64,                // User's share amount
    pub deposited_sol: u64,         // How much SOL deposited (for tracking)
    pub last_claim_round: u64,      // Last round rewards claimed
}
```

### 1.3 Define PDAs

**Pool Authority PDA:**
```rust
seeds = [b"pool_authority", strategy.to_bytes()], bump
```

**User Deposit PDA:**
```rust
seeds = [b"user_deposit", pool.key(), user.key()], bump
```

**ORE Miner PDA (managed by ORE program):**
```rust
// From ORE program:
seeds = [b"miner", pool_authority.key()], program_id = ORE_PROGRAM_ID
```

---

## Phase 2: Core Instructions

### 2.1 Initialize Pool

**Instruction:** `initialize_pool`

**Accounts:**
- `authority` (signer) - Admin who initializes
- `pool` (init) - Pool state account
- `pool_authority` (PDA) - Pool's miner authority
- `fee_collector` - Where fees go
- `system_program`

**Logic:**
1. Create pool state account
2. Set strategy (25 or 18 squares)
3. Initialize counters to 0
4. Set fee collector
5. Set fee to 200 basis points (2%)

**No ORE CPI needed** - just creates pool state.

### 2.2 Deposit

**Instruction:** `deposit`

**Accounts:**
- `user` (signer, mut) - User depositing
- `pool` (mut) - Pool state
- `user_deposit` (init_if_needed, mut) - User's deposit account
- `system_program`

**Logic:**
1. Transfer SOL from user to pool authority PDA
2. Calculate shares: `shares = (amount * total_shares) / total_sol_current`
   - If first deposit: `shares = amount` (1:1 ratio)
3. Mint shares to user
4. Update pool totals
5. Update user deposit record

**Parameters:**
- `amount: u64` - SOL to deposit (lamports)

**Validation:**
- `amount > 0`
- Pool not paused
- `amount >= MINIMUM_DEPOSIT` (e.g., 0.01 SOL)

### 2.3 Mine (Deploy to Round)

**Instruction:** `mine`

**Accounts:**
- `payer` (signer, mut) - Pays transaction fees (bot wallet)
- `pool` (mut) - Pool state
- `pool_authority` (PDA) - Pool's authority (miner)
- `ore_board` - ORE board account
- `ore_round` - ORE round account
- `ore_miner` - Pool's ORE miner account (will be created on first deploy)
- `ore_automation` - ORE automation PDA for pool_authority
- `entropy_var` - Entropy randomness account
- `entropy_program` - Entropy program
- `ore_program` - ORE program
- `system_program`

**Logic:**
1. Check pool not paused
2. Get current round from board
3. Read round data to analyze square deployments
4. Calculate bet size:
   ```rust
   let bet_per_round = pool.total_sol_current / 20; // 5% of pool per round
   let bet_per_square = bet_per_round / num_squares;
   ```
5. Select squares based on strategy:
   - 25-square: All squares `[true; 25]`
   - 18-square: Call `select_18_least_crowded(round)` function
6. Build `ore_api::deploy()` instruction
7. Make CPI call with pool_authority as signer
8. Update `pool.last_round_id`

**Square Selection for 18-Square Strategy:**
```rust
fn select_18_least_crowded(round: &Round) -> [bool; 25] {
    let mut squares_by_deployment: Vec<(usize, u64)> = round
        .deployed
        .iter()
        .enumerate()
        .map(|(i, &d)| (i, d))
        .collect();

    // Sort by deployment (ascending = least crowded first)
    squares_by_deployment.sort_by_key(|&(_, d)| d);

    let mut selected = [false; 25];
    for i in 0..18 {
        selected[squares_by_deployment[i].0] = true;
    }
    selected
}
```

**CPI Call:**
```rust
use anchor_lang::prelude::*;
use ore_api;

let deploy_ix = ore_api::sdk::deploy(
    ctx.accounts.payer.key(),
    ctx.accounts.pool_authority.key(),
    bet_per_square,
    board.round_id,
    squares,
);

let authority_seeds = &[
    b"pool_authority",
    &pool.strategy.to_bytes(),
    &[pool.authority_bump],
];
let signer_seeds = &[&authority_seeds[..]];

anchor_lang::solana_program::program::invoke_signed(
    &deploy_ix,
    &[
        ctx.accounts.payer.to_account_info(),
        ctx.accounts.pool_authority.to_account_info(),
        ctx.accounts.ore_automation.to_account_info(),
        ctx.accounts.ore_board.to_account_info(),
        ctx.accounts.ore_miner.to_account_info(),
        ctx.accounts.ore_round.to_account_info(),
        ctx.accounts.system_program.to_account_info(),
        ctx.accounts.entropy_var.to_account_info(),
        ctx.accounts.entropy_program.to_account_info(),
    ],
    signer_seeds,
)?;
```

### 2.4 Checkpoint

**Instruction:** `checkpoint`

**Accounts:**
- `payer` (signer, mut)
- `pool` (mut)
- `pool_authority` (PDA)
- `ore_miner` (mut)
- `ore_round` (mut)
- `ore_program`
- Additional ORE accounts...

**Logic:**
1. Verify round is complete (slot_hash is set)
2. Make CPI to `ore_api::checkpoint(pool_authority, round_id)`
3. Updates ore_miner account with rewards

**CPI Call:**
```rust
let checkpoint_ix = ore_api::sdk::checkpoint(
    ctx.accounts.pool_authority.key(),
    ctx.accounts.pool_authority.key(),
    round_id,
);

anchor_lang::solana_program::program::invoke_signed(
    &checkpoint_ix,
    &[/* accounts */],
    signer_seeds,
)?;
```

### 2.5 Claim Rewards

**Instruction:** `claim_rewards`

**Accounts:**
- `payer` (signer, mut)
- `pool` (mut)
- `pool_authority` (PDA, mut)
- `fee_collector` (mut)
- `ore_miner` (mut)
- `pool_ore_token_account` (mut) - Pool's ORE token account
- `fee_ore_token_account` (mut) - Fee collector's ORE token account
- `ore_mint` - ORE token mint
- `ore_treasury` - ORE treasury
- `ore_treasury_tokens` - ORE treasury token account
- `ore_program`
- `token_program`
- `system_program`

**Logic:**
1. Read current miner account state (before claiming)
2. Make CPI to `ore_api::claim_sol(pool_authority)`
   - SOL goes to pool_authority
3. Make CPI to `ore_api::claim_ore(pool_authority)`
   - ORE goes to pool's token account
4. Read miner account state (after claiming)
5. Calculate claimed amounts:
   ```rust
   let sol_claimed = pool_authority.lamports() - initial_balance;
   let ore_claimed = pool_ore_account.amount - initial_ore;
   ```
6. Take 2% management fee:
   ```rust
   let fee_sol = sol_claimed * 200 / 10000;
   let fee_ore = ore_claimed * 200 / 10000;
   ```
7. Transfer fees to fee_collector
8. Update pool state:
   ```rust
   pool.total_sol_current += sol_claimed - fee_sol;
   pool.total_ore_claimed += ore_claimed - fee_ore;
   ```

### 2.6 Withdraw

**Instruction:** `withdraw`

**Accounts:**
- `user` (signer, mut)
- `pool` (mut)
- `pool_authority` (mut)
- `user_deposit` (mut)
- `user_ore_token_account` (mut)
- `pool_ore_token_account` (mut)
- `token_program`
- `system_program`

**Parameters:**
- `shares: u64` - How many shares to redeem

**Logic:**
1. Calculate user's portion:
   ```rust
   let sol_amount = (shares * pool.total_sol_current) / pool.total_shares;
   let ore_amount = (shares * pool.total_ore_claimed) / pool.total_shares;
   ```
2. Transfer SOL from pool_authority to user
3. Transfer ORE from pool token account to user token account
4. Burn user's shares:
   ```rust
   user_deposit.shares -= shares;
   pool.total_shares -= shares;
   pool.total_sol_current -= sol_amount;
   pool.total_ore_claimed -= ore_amount;
   ```

**Validation:**
- User has enough shares
- `shares > 0`
- Pool has enough liquidity

### 2.7 Pause / Unpause

**Instruction:** `pause` / `unpause`

**Accounts:**
- `authority` (signer) - Pool admin
- `pool` (mut)

**Logic:**
- Set `pool.paused = true/false`
- When paused: deposits and mining disabled, withdrawals still work

---

## Phase 3: Bot Integration

### 3.1 Mining Bot Requirements

**Create new file:** `/mining-pools/bot/pool_miner.rs`

**Bot responsibilities:**
1. Monitor current round from ORE board
2. When new round starts:
   - Call `mine` instruction for Pool A
   - Call `mine` instruction for Pool B
3. When round ends:
   - Wait for slot_hash to be set
   - Call `checkpoint` for Pool A
   - Call `checkpoint` for Pool B
4. After checkpointing:
   - Call `claim_rewards` for Pool A
   - Call `claim_rewards` for Pool B

**Timing:**
- Deploy at 5-10s remaining (like current strategy)
- Checkpoint immediately after round ends
- Claim rewards once per hour or after every round

**Bot wallet:**
- Separate wallet from pool
- Pays transaction fees
- Doesn't need large balance (~0.1 SOL)

### 3.2 Bot Code Structure

```rust
// Pseudocode
async fn pool_mining_loop() {
    loop {
        let board = get_board().await;
        let pool_a_state = get_pool_state(POOL_A_ADDRESS).await;
        let pool_b_state = get_pool_state(POOL_B_ADDRESS).await;

        // Wait for optimal deploy window
        wait_for_deploy_window(10, 5).await;

        // Mine for both pools
        mine_pool(POOL_A_ADDRESS, board.round_id).await;
        mine_pool(POOL_B_ADDRESS, board.round_id).await;

        // Wait for round to end
        wait_for_round_end().await;

        // Checkpoint both pools
        checkpoint_pool(POOL_A_ADDRESS, board.round_id).await;
        checkpoint_pool(POOL_B_ADDRESS, board.round_id).await;

        // Claim rewards
        claim_pool_rewards(POOL_A_ADDRESS).await;
        claim_pool_rewards(POOL_B_ADDRESS).await;
    }
}
```

---

## Phase 4: Frontend Integration

### 4.1 Required Smart Contract Queries

**Get Pool State:**
```typescript
const poolA = await program.account.pool.fetch(POOL_A_ADDRESS);
```

**Get User Deposit:**
```typescript
const [userDepositPda] = PublicKey.findProgramAddressSync(
  [Buffer.from("user_deposit"), poolAddress.toBuffer(), wallet.publicKey.toBuffer()],
  program.programId
);
const userDeposit = await program.account.userDeposit.fetch(userDepositPda);
```

**Get Pool's ORE Miner Account:**
```typescript
const [poolAuthorityPda] = PublicKey.findProgramAddressSync(
  [Buffer.from("pool_authority"), strategyBytes],
  program.programId
);

const [oreMinerPda] = PublicKey.findProgramAddressSync(
  [Buffer.from("miner"), poolAuthorityPda.toBuffer()],
  ORE_PROGRAM_ID
);

const oreMiner = await oreProgram.account.miner.fetch(oreMinerPda);
```

### 4.2 User Actions

**Deposit to Pool:**
```typescript
await program.methods
  .deposit(new BN(amount))
  .accounts({
    user: wallet.publicKey,
    pool: poolAddress,
    userDeposit: userDepositPda,
    systemProgram: SystemProgram.programId,
  })
  .rpc();
```

**Withdraw from Pool:**
```typescript
await program.methods
  .withdraw(new BN(shares))
  .accounts({
    user: wallet.publicKey,
    pool: poolAddress,
    poolAuthority: poolAuthorityPda,
    userDeposit: userDepositPda,
    userOreTokenAccount: userOreAta,
    poolOreTokenAccount: poolOreAta,
    tokenProgram: TOKEN_PROGRAM_ID,
    systemProgram: SystemProgram.programId,
  })
  .rpc();
```

### 4.3 Real-time Stats

**Calculate User's Share Value:**
```typescript
const userShareValue = {
  sol: (userDeposit.shares * poolState.totalSolCurrent) / poolState.totalShares,
  ore: (userDeposit.shares * poolState.totalOreClaimed) / poolState.totalShares,
};
```

**Calculate Pool APR:**
```typescript
// Track pool performance over time
// APR = (total_winnings / total_deposits) * (365 / days_elapsed) * 100
```

---

## Phase 5: Testing Plan

### 5.1 Devnet Testing

**Prerequisites:**
- Deploy pool contract to devnet
- Get devnet SOL from faucet
- Verify ORE v3 is running on devnet

**Test Scenarios:**

1. **Single User Deposit/Withdraw:**
   - Deposit 1 SOL
   - Mine 5 rounds
   - Verify rewards accumulate
   - Withdraw all
   - Verify correct amounts received

2. **Multi-User Proportional Distribution:**
   - User A deposits 1 SOL
   - User B deposits 2 SOL
   - Mine 10 rounds
   - Verify rewards distributed 1:2 ratio
   - Both users withdraw
   - Verify correct proportions

3. **Edge Cases:**
   - Deposit minimum amount (0.01 SOL)
   - Withdraw partial shares
   - Deposit/withdraw during same round
   - Multiple deposits from same user
   - Withdraw all shares (should close user_deposit account)

4. **Bot Functionality:**
   - Run bot for 24 hours
   - Verify it mines every round
   - Verify checkpoints succeed
   - Verify rewards claimed
   - Check for failed transactions

5. **Strategy Verification:**
   - Pool A should win 90%+ of rounds (25 squares)
   - Pool B should win 70%+ of rounds (18 squares)
   - Track cost per ORE for each pool

### 5.2 Mainnet Testing (Beta)

**Before full launch:**
- Deploy to mainnet
- Seed with 10 SOL personal capital
- Invite 5-10 trusted users
- Run for 1 week
- Monitor closely
- Fix any issues

---

## Phase 6: Security Audit Checklist

### 6.1 Smart Contract Security

**Check for:**
- [ ] Integer overflow/underflow (use checked math)
- [ ] Reentrancy (not applicable in Solana, but check CPI ordering)
- [ ] Unauthorized access (verify signers)
- [ ] Share calculation precision (avoid rounding errors)
- [ ] PDA derivation correctness
- [ ] CPI signer seeds correctness
- [ ] Emergency pause functionality
- [ ] Withdrawal DoS (users can always withdraw)
- [ ] Fee calculation bounds (max 10%?)

### 6.2 Economic Security

**Verify:**
- [ ] Share calculation doesn't allow gaming
- [ ] Fee extraction is correct (exactly 2%, no more)
- [ ] Can't deposit 0 and get shares
- [ ] Can't withdraw more than owned
- [ ] Rounding errors favor pool, not attacker
- [ ] No front-running opportunities

### 6.3 Operational Security

**Ensure:**
- [ ] Bot wallet secured (not pool admin)
- [ ] Fee collector wallet secured
- [ ] Admin key for pause is secure
- [ ] Private keys never committed to git
- [ ] RPC endpoint has rate limits
- [ ] Monitoring/alerting for bot failures

---

## Phase 7: Launch Checklist

### 7.1 Pre-Launch

- [ ] Smart contract audited
- [ ] 100+ successful rounds on devnet
- [ ] Frontend tested with real users
- [ ] Bot runs 24/7 without issues
- [ ] Documentation complete
- [ ] Terms of service / disclaimers ready
- [ ] Legal consultation complete

### 7.2 Launch Day

- [ ] Deploy contracts to mainnet
- [ ] Initialize Pool A and Pool B
- [ ] Seed initial liquidity (10 SOL each)
- [ ] Start bot
- [ ] Deploy frontend
- [ ] Announce to ORE community

### 7.3 Post-Launch

- [ ] Monitor pools 24/7 for first week
- [ ] Track user deposits
- [ ] Verify rewards distributed correctly
- [ ] Gather user feedback
- [ ] Fix any issues immediately

---

## Open Questions to Resolve

### Q1: Automation Account

**Question:** Do we need to initialize the automation PDA for the pool authority?

**Research needed:**
- Check if ORE program auto-creates automation account
- Or if we need to call `automate` instruction first
- Look at existing miners' automation accounts

### Q2: Account Rent

**Question:** How much SOL needed for rent-exempt accounts?

**Calculate:**
- Pool state account size
- User deposit account size
- Miner account (created by ORE)
- Total rent per pool
- Total rent per user

### Q3: Transaction Fees for CPI

**Question:** Does pool_authority need SOL for CPI transaction fees?

**Test:**
- Does `invoke_signed` deduct from PDA's balance?
- Or from `payer` account?
- Current understanding: `payer` pays, but verify

### Q4: ORE Token Account Creation

**Question:** How to properly create pool's ORE token account?

**Research:**
- Use Associated Token Account (ATA)?
- Or custom token account?
- Who pays rent?
- When to create (in initialize or first claim)?

### Q5: Devnet Availability

**Question:** Is ORE v3 running on devnet?

**Check:**
- Are there active rounds on devnet?
- Can we mine on devnet?
- Or do we need to test on mainnet directly?

---

## Implementation Timeline

**Total estimated time:** 3-4 weeks (with proper testing)

### Week 1: Smart Contract Development
- Day 1-2: Set up project structure, dependencies
- Day 3-4: Implement core instructions (initialize, deposit, withdraw)
- Day 5-7: Implement mining instructions (mine, checkpoint, claim)

### Week 2: Testing & Bot Integration
- Day 8-9: Write integration tests
- Day 10-11: Build mining bot
- Day 12-14: Test on devnet extensively

### Week 3: Frontend & Documentation
- Day 15-17: Build frontend UI
- Day 18-19: Integrate wallet connection
- Day 20-21: Write user documentation

### Week 4: Audit & Launch
- Day 22-25: Security audit & fixes
- Day 26-27: Beta testing with trusted users
- Day 28: Launch to public

---

## Success Metrics

### Technical Success
- [ ] 0 failed transactions in 100 rounds
- [ ] Rewards distributed with <0.01% error
- [ ] Bot uptime >99.9%
- [ ] All user withdrawals successful

### Business Success
- [ ] 20+ depositors in first week
- [ ] 50+ SOL TVL in first month
- [ ] Pool A win rate >90%
- [ ] Pool B win rate >70%
- [ ] Cost per ORE <$800

---

## Next Immediate Steps

1. **Resolve open questions** (automation account, rent, fees)
2. **Set up Anchor project** with correct dependencies
3. **Implement Pool and UserDeposit accounts** with proper validation
4. **Implement Initialize and Deposit instructions** (no ORE CPI needed)
5. **Test deposit/withdraw flow** without mining first
6. **Then implement Mine instruction** with proper CPI
7. **Test on devnet**

This plan is based on actual ORE SDK capabilities and will result in working code.
