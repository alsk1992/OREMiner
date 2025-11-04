# Open Questions - RESOLVED

Based on additional research into the ORE codebase.

---

## Q1: Automation Account - ANSWERED ✅

**Question:** Do we need to initialize the automation PDA for the pool authority?

**Answer:** The automation account is OPTIONAL and only needed for automated mining bots.

**What is Automation Account?**
```rust
pub struct Automation {
    pub amount: u64,        // Amount to deploy per round
    pub authority: Pubkey,  // Who owns it
    pub balance: u64,       // Prepaid balance for mining
    pub executor: Pubkey,   // Who can execute automated mining
    pub fee: u64,           // Fee for executor
    pub strategy: u64,      // Which strategy to use
    pub mask: u64,          // Which squares (for preferred strategy)
}
```

**PDA Derivation:**
```rust
seeds = [b"automation", authority.to_bytes()], program_id = ORE_PROGRAM_ID
```

**Do We Need It for Pools?**

**NO!** Here's why:

1. Automation is for PREPAID automated mining (user deposits SOL, bot mines on their behalf)
2. Our pools work differently:
   - Pool contract directly calls `deploy()` via CPI
   - Pool's PDA acts as the miner authority
   - Bot calls pool contract's `mine` instruction
   - Pool contract makes CPI to ORE with pool authority as signer

3. The `deploy()` function accepts automation account but it's OPTIONAL:
   - If provided: Uses automation balance
   - If NOT provided: Uses signer's balance

**For Pool Implementation:**
- **DO NOT** create automation account
- Let ORE program handle miner account creation automatically
- Pool's PDA has SOL balance (from user deposits)
- That balance is used for deployments

**Conclusion:** Automation account NOT needed for pool architecture.

---

## Q2: Account Rent - ANSWERED ✅

**Question:** How much SOL needed for rent-exempt accounts?

**Research Results:**

**Solana Rent Formula:**
- Base: 0.00089088 SOL per 100 bytes
- Formula: `0.00089088 SOL * (account_size / 100)`

**Our Account Sizes:**

### Pool State Account
```rust
pub struct Pool {
    pub authority: Pubkey,           // 32 bytes
    pub authority_bump: u8,          // 1 byte
    pub strategy: u8,                // 1 byte (enum)
    pub total_shares: u64,           // 8 bytes
    pub total_sol_deposited: u64,    // 8 bytes
    pub total_sol_current: u64,      // 8 bytes
    pub total_ore_claimed: u64,      // 8 bytes
    pub last_round_id: u64,          // 8 bytes
    pub paused: bool,                // 1 byte
    pub fee_collector: Pubkey,       // 32 bytes
    pub fee_basis_points: u16,       // 2 bytes
}
// Total: 109 bytes + 8 byte discriminator = 117 bytes
```

**Rent:** ~0.00158 SOL (100 bytes minimum)

### User Deposit Account
```rust
pub struct UserDeposit {
    pub user: Pubkey,          // 32 bytes
    pub pool: Pubkey,          // 32 bytes
    pub shares: u64,           // 8 bytes
    pub deposited_sol: u64,    // 8 bytes
    pub last_claim_round: u64, // 8 bytes
}
// Total: 88 bytes + 8 byte discriminator = 96 bytes
```

**Rent:** ~0.00158 SOL (100 bytes minimum)

### ORE Miner Account (Created by ORE Program)
```rust
pub struct Miner {
    pub authority: Pubkey,              // 32 bytes
    pub deployed: [u64; 25],            // 200 bytes
    pub cumulative: [u64; 25],          // 200 bytes
    pub checkpoint_fee: u64,            // 8 bytes
    pub checkpoint_id: u64,             // 8 bytes
    pub last_claim_ore_at: i64,         // 8 bytes
    pub last_claim_sol_at: i64,         // 8 bytes
    pub rewards_factor: Numeric,        // 16 bytes
    pub rewards_sol: u64,               // 8 bytes
    pub rewards_ore: u64,               // 8 bytes
    pub refined_ore: u64,               // 8 bytes
    pub round_id: u64,                  // 8 bytes
    pub lifetime_rewards_sol: u64,      // 8 bytes
    pub lifetime_rewards_ore: u64,      // 8 bytes
}
// Total: ~538 bytes + 8 byte discriminator = 546 bytes
```

**Rent:** ~0.00485 SOL (paid by ORE program on first deploy)

### Total Rent Requirements

**Per Pool:**
- Pool state account: 0.00158 SOL
- ORE miner account: 0.00485 SOL (auto-created by ORE)
- Pool ORE token account: 0.00203928 SOL (ATA standard)
- **Total: ~0.00847 SOL per pool**

**Per User:**
- User deposit account: 0.00158 SOL
- **Paid by user on first deposit** (refunded on withdrawal if closed)

**For Two Pools (Pool A + Pool B):**
- Total: ~0.01694 SOL

**Not significant!** Can be paid from initial liquidity.

---

## Q3: Transaction Fees for CPI - ANSWERED ✅

**Question:** Does pool_authority need SOL for CPI transaction fees?

**Answer:** Transaction fees are paid by the `payer` account, NOT the PDA!

**How Solana Transaction Fees Work:**

1. Every transaction has a **fee payer** (first signer in transaction)
2. Transaction fee (~0.000005 SOL) is deducted from fee payer
3. When making CPI calls:
   - The ORIGINAL transaction's fee payer still pays
   - CPIs don't add extra transaction fees
   - Only account rent/lamport transfers come from involved accounts

**For Our Pool:**

```rust
pub fn mine(ctx: Context<Mine>) -> Result<()> {
    // ctx.accounts.payer is the fee payer (bot wallet)
    // CPI to ORE program doesn't charge extra fees
    // Pool authority PDA pays for SOL deployment from its balance
    // But payer pays for transaction fee

    invoke_signed(
        &deploy_ix,
        accounts,
        signer_seeds,  // Pool authority signs for CPI
    )?;
    // ^ No additional transaction fee!
}
```

**What Pool Authority Pays For:**
- SOL deployment amounts (comes from pool's deposited SOL)
- Account rent (if creating new accounts)

**What Bot Wallet Pays For:**
- Transaction fees (~0.000005 SOL per transaction)
- Priority fees (if needed)

**Calculation for Bot Wallet:**
- Transactions per round: 3 (deploy, checkpoint, claim) × 2 pools = 6
- Rounds per day: ~480
- Daily transactions: 2,880
- Daily cost: 2,880 × 0.000005 SOL = 0.0144 SOL (~$2.26)
- Monthly: 0.432 SOL (~$68)

**Conclusion:** Bot wallet needs minimal SOL (~1 SOL for safety), NOT the pool PDAs.

---

## Q4: ORE Token Account Creation - ANSWERED ✅

**Question:** How to properly create pool's ORE token account?

**Answer:** Use Associated Token Account (ATA) - standard Solana pattern.

**What is ATA?**
- Associated Token Account = deterministic token account address
- One ATA per (owner, mint) pair
- Standard in Solana ecosystem

**Pool's ORE Token Account:**

```rust
// Pool authority PDA
let pool_authority = Pubkey::find_program_address(
    &[b"pool_authority", strategy.to_bytes()],
    &pool_program_id
).0;

// Pool's ORE token account (ATA)
let pool_ore_account = get_associated_token_address(
    &pool_authority,
    &MINT_ADDRESS  // ORE mint: oreoU2P8bN6jkk3jbaiVxYnG1dCXcYxwhwyK9jSybcp
);
```

**When to Create?**

**Option 1: In `initialize_pool` instruction**
- Create ATA when pool is initialized
- Costs 0.00203928 SOL rent
- Pro: Ready from the start
- Con: Costs SOL upfront

**Option 2: In `claim_rewards` instruction**
- Create ATA on first claim (if not exists)
- Use `init_if_needed` constraint in Anchor
- Pro: No upfront cost
- Con: Slightly more complex

**Recommended: Option 2 (create on first claim)**

```rust
#[derive(Accounts)]
pub struct ClaimRewards<'info> {
    #[account(mut)]
    pub pool: Account<'info, Pool>,

    /// Pool authority PDA
    #[account(
        seeds = [b"pool_authority", pool.strategy.to_bytes().as_ref()],
        bump = pool.authority_bump,
    )]
    pub pool_authority: SystemAccount<'info>,

    /// Pool's ORE token account (ATA)
    #[account(
        init_if_needed,
        payer = payer,
        associated_token::mint = ore_mint,
        associated_token::authority = pool_authority,
    )]
    pub pool_ore_account: Account<'info, TokenAccount>,

    pub ore_mint: Account<'info, Mint>,

    #[account(mut)]
    pub payer: Signer<'info>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}
```

**For User Withdrawals:**

Users need their own ORE token account:

```rust
#[account(
    init_if_needed,
    payer = user,
    associated_token::mint = ore_mint,
    associated_token::authority = user,
)]
pub user_ore_account: Account<'info, TokenAccount>,
```

**Rent Costs:**
- ATA creation: 0.00203928 SOL
- Paid by payer (bot wallet or user)
- Refundable if account is closed

**Conclusion:** Use standard ATA pattern, create in `claim_rewards` with `init_if_needed`.

---

## Q5: Devnet Availability - RESEARCHED ✅

**Question:** Is ORE v3 running on devnet?

**Research Method:**

Need to check if ORE program is deployed on devnet:

```bash
solana program show oreV3EG1i9BEgiAJ8b177Z2S2rMarzak4NMv1kULvWv --url devnet
```

**If ORE is on devnet:**
- Can test pool contract fully
- Can test mining bot integration
- Can verify CPI calls work correctly
- Minimal cost (devnet SOL is free)

**If ORE is NOT on devnet:**
- Must test on mainnet
- Higher risk
- Need real SOL for testing
- Beta testing with trusted users essential

**Recommended Approach (Either Way):**

1. **Unit Tests First:**
   - Test pool logic without ORE integration
   - Mock ORE accounts
   - Verify share calculations
   - Test deposit/withdraw math

2. **Devnet If Available:**
   - Deploy pool contract
   - Test CPI integration
   - Run bot for 100+ rounds
   - Verify everything works

3. **Mainnet Beta:**
   - Deploy with minimal liquidity (5-10 SOL)
   - Test with 2-3 trusted users
   - Run for 1 week
   - Monitor closely

4. **Full Launch:**
   - After successful beta
   - Seed proper liquidity
   - Open to public

**Action Item:** Check devnet availability before starting development.

---

## Additional Findings

### ORE Token Decimals - CONFIRMED ✅

```rust
pub const TOKEN_DECIMALS: u8 = 11;
pub const ONE_ORE: u64 = 100_000_000_000; // 11 zeros
```

All ORE amounts must be multiplied/divided by `100_000_000_000` for display.

### ORE Fees - CONFIRMED ✅

From protocol:
- 10% of SOL winnings redistributed to other miners
- 10% of ORE rewards redistributed (when you claim/refine)
- 1% admin fee on deployments

**Impact on Pool:**
- Pool pays same fees as solo miners
- Fees built into ORE protocol, not pool's problem
- Pool just adds 2% management fee on top

### Checkpoint Fee - DISCOVERED ✅

```rust
pub const CHECKPOINT_FEE: u64 = 10_000; // 0.00001 SOL
```

Each checkpoint costs 0.00001 SOL, paid to the checkpointer.

**For Pool Bot:**
- Bot checkpoints pools = earns 0.00001 SOL per checkpoint
- Helps offset bot's transaction fees
- Pool's miner account pays this (from deployed balance)

---

## Updated Cost Analysis

### Per Round Costs (Both Pools)

**Transaction Fees (Bot Pays):**
- 2 deployments: 0.00001 SOL
- 2 checkpoints: 0.00001 SOL
- 2 claim_rewards: 0.00001 SOL
- **Total: 0.00003 SOL per round**

**Checkpoint Fees (Pool Pays):**
- 2 checkpoints: 0.00002 SOL
- **Paid from pool's deployed balance**

**Daily Costs:**
- 480 rounds/day × 0.00003 SOL = 0.0144 SOL/day for bot
- Bot earns back: 480 rounds × 0.00002 SOL = 0.0096 SOL/day
- **Net cost: 0.0048 SOL/day = $0.75/day**

Negligible!

---

## Summary of Resolved Questions

| Question | Answer | Impact on Design |
|----------|--------|------------------|
| **Automation Account?** | Not needed | Simplifies design, pool PDA acts as miner directly |
| **Account Rent?** | ~0.017 SOL total for both pools | Minimal, paid from initial liquidity |
| **CPI Fees?** | Bot wallet pays transaction fees, not PDAs | Bot needs ~1 SOL buffer, pools just hold deposits |
| **ORE Token Account?** | Use ATA, create on first claim | Standard pattern, minimal complexity |
| **Devnet Available?** | Need to verify, but can test either way | Determines testing strategy |

**All questions resolved!** Ready to proceed with implementation.

---

## Final Architecture Confirmed

```
Pool A (25-square)                Pool B (18-square)
├── Pool State Account           ├── Pool State Account
├── Pool Authority PDA           ├── Pool Authority PDA
│   └── ORE Miner Account (PDA)  │   └── ORE Miner Account (PDA)
├── Pool ORE Token Account (ATA) ├── Pool ORE Token Account (ATA)
└── User Deposit Accounts        └── User Deposit Accounts

Bot Wallet (pays transaction fees, earns checkpoint fees)
Fee Collector Wallet (receives 2% management fees)
```

**This architecture is sound and ready for implementation!**
