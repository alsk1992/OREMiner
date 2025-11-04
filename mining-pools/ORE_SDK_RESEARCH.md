# ORE SDK Research - What's Actually Possible

## Overview

This document contains thorough research of the ORE v3 SDK to understand exactly what's possible for building trustless mining pools.

---

## Key ORE SDK Functions

### 1. Deploy Function

```rust
pub fn deploy(
    signer: Pubkey,      // Who pays for the transaction (signs)
    authority: Pubkey,   // The miner authority (miner account owner)
    amount: u64,         // Amount PER SQUARE in lamports
    round_id: u64,       // Which round to deploy to
    squares: [bool; 25], // Which squares to deploy to
) -> Instruction
```

**Required Accounts:**
1. `signer` (mut, signer) - Pays for transaction
2. `authority` (mut) - The miner authority
3. `automation_pda` (mut) - PDA derived from authority
4. `board_pda` (mut) - Global board state
5. `miner_pda` (mut) - Miner account (PDA from authority)
6. `round_pda` (mut) - Round account
7. `system_program` (readonly)
8. `entropy_var` (mut) - Entropy randomness account
9. `entropy_program` (readonly)

**Critical Discovery:**
- The `authority` parameter determines WHO the miner account belongs to
- The `signer` just pays for the transaction
- This means: **A pool PDA CAN be the authority!**

### 2. Checkpoint Function

```rust
pub fn checkpoint(
    signer: Pubkey,
    authority: Pubkey,
    round_id: u64,
) -> Instruction
```

**Purpose:** Finalizes a round and calculates rewards for the miner account.

**Required Accounts:**
1. `signer` (mut, signer)
2. `authority` (mut)
3. `miner_pda` (mut)
4. `round_pda` (mut)
5. System accounts...

### 3. Claim SOL Function

```rust
pub fn claim_sol(signer: Pubkey) -> Instruction
```

**Purpose:** Claims accumulated SOL rewards from miner account.

**Key Point:** Claims SOL from `miner_pda(signer)` - the signer's miner account.

### 4. Claim ORE Function

```rust
pub fn claim_ore(signer: Pubkey) -> Instruction
```

**Purpose:** Claims accumulated ORE rewards (refines unrefined ORE with 10% fee, transfers to signer's token account).

---

## PDA Derivation Functions

### Miner PDA
```rust
pub fn miner_pda(authority: Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[MINER, &authority.to_bytes()], &ORE_PROGRAM_ID)
}
```

**Seeds:** `["miner", authority_pubkey]`

### Board PDA
```rust
pub fn board_pda() -> (Pubkey, u8) {
    Pubkey::find_program_address(&[BOARD], &ORE_PROGRAM_ID)
}
```

**Seeds:** `["board"]` (global, singleton)

### Round PDA
```rust
pub fn round_pda(id: u64) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[ROUND, &id.to_le_bytes()], &ORE_PROGRAM_ID)
}
```

**Seeds:** `["round", round_id_bytes]`

---

## Account Structures

### Miner Account
```rust
pub struct Miner {
    pub authority: Pubkey,          // Owner of this miner account
    pub deployed: [u64; 25],        // How much deployed per square (current round)
    pub rewards_sol: u64,           // Claimable SOL rewards
    pub rewards_ore: u64,           // Unrefined ORE (earns 150% APR!)
    pub refined_ore: u64,           // Refined ORE (claimable)
    pub checkpoint_id: u64,         // Last checkpointed round
    pub round_id: u64,              // Last deployed round
    // ... more fields
}
```

**Key Insights:**
- Each authority has ONE miner account (PDA)
- Rewards accumulate in the miner account
- Unrefined ORE earns 150% APR automatically

### Round Account
```rust
pub struct Round {
    pub id: u64,
    pub deployed: [u64; 25],        // Total SOL deployed per square
    pub slot_hash: [u8; 32],        // RNG seed (set when round ends)
    pub count: [u64; 25],           // Number of miners per square
    pub motherlode: u64,            // Motherlode ORE prize
    pub top_miner: Pubkey,          // Who gets top_miner_reward
    pub top_miner_reward: u64,      // Top miner ORE prize
    pub total_deployed: u64,        // Total SOL in round
    pub total_winnings: u64,        // Total SOL won
    // ... more fields
}
```

### Board Account
```rust
pub struct Board {
    pub round_id: u64,      // Current round number
    pub start_slot: u64,    // When current round started
    pub end_slot: u64,      // When current round ends
}
```

---

## Critical Discoveries

### ✅ Can a Pool PDA Act as a Miner?

**YES!** Here's how:

1. Pool contract creates a PDA (e.g., `pool_authority_pda`)
2. Pool contract makes CPI call to `ore_api::deploy()` with:
   - `signer`: The user who invoked the pool instruction (pays fees)
   - `authority`: **The pool's PDA** (owns the miner account)
   - `amount`: Calculated from pool's total balance
   - `squares`: Strategy (25 or 18 squares)

3. ORE program creates/updates `miner_pda(pool_authority_pda)`
4. Rewards accumulate in the pool's miner account
5. Pool contract can claim rewards via CPI and distribute to participants

### ✅ Can Pool Distribute Rewards?

**YES!** Process:

1. After round ends, pool calls `checkpoint()` via CPI
2. Pool calls `claim_sol()` via CPI - SOL goes to pool's authority
3. Pool calls `claim_ore()` via CPI - ORE goes to pool's token account
4. Pool distributes proportionally based on shares

### ✅ Can Pool Handle Multiple Users?

**YES!** Architecture:

```
Pool Contract
├── Pool Authority PDA (acts as miner)
│   └── Controls: miner_pda(pool_authority)
├── Pool State Account
│   ├── total_shares: u64
│   ├── total_sol_balance: u64
│   ├── total_ore_earned: u64
│   └── strategy: PoolStrategy
└── User Deposit Accounts (PDAs per user)
    ├── user_pubkey: Pubkey
    ├── shares: u64
    ├── deposited_sol: u64
    └── last_claim_checkpoint: u64
```

---

## What's NOT Possible

### ❌ Can't Customize Reward Distribution During Mining

- ORE program handles all reward calculations
- You can't intercept or modify how rewards are distributed within a round
- Pool can only distribute AFTER claiming from ORE program

### ❌ Can't Create "Sub-Miners" Per User

- Each user can't have their own miner account within the pool
- Pool must aggregate all users into ONE miner account
- Distribution happens at the pool contract level

### ❌ Can't Deploy Different Strategies Simultaneously

- One miner account = one deployment per round
- Pool A (25 squares) and Pool B (18 squares) need SEPARATE pool PDAs
- Each pool = separate miner account

---

## Architecture Design

Based on research, here's what's actually possible:

### Two Separate Pools

**Pool A - "The Banker" (25 Squares)**
- Pool PDA: `pool_a_authority`
- Miner Account: `miner_pda(pool_a_authority)`
- Strategy: Deploy to all 25 squares
- Users deposit SOL, get shares in Pool A

**Pool B - "The Grinder" (18 Squares)**
- Pool PDA: `pool_b_authority`
- Miner Account: `miner_pda(pool_b_authority)`
- Strategy: Deploy to 18 least crowded squares
- Users deposit SOL, get shares in Pool B

### Required Smart Contract Instructions

1. **Initialize Pool**
   - Creates pool state account
   - Sets up pool authority PDA
   - Defines strategy (25 or 18 squares)

2. **Deposit**
   - User deposits SOL
   - Receives shares proportional to deposit
   - Updates pool total balance

3. **Mine** (called by bot/admin)
   - Reads current round from ORE board
   - Calculates bet size from pool balance
   - Selects squares based on strategy
   - CPI to `ore_api::deploy()` with pool authority as miner
   - Pays transaction fees from pool or designated wallet

4. **Checkpoint** (called by bot/admin)
   - CPI to `ore_api::checkpoint()` after round ends
   - Updates pool's miner account with rewards

5. **Claim Rewards** (called by bot/admin)
   - CPI to `ore_api::claim_sol()`
   - CPI to `ore_api::claim_ore()`
   - Updates pool's reward balances
   - Takes 2% management fee

6. **Withdraw**
   - User burns shares
   - Receives proportional SOL + ORE rewards
   - Updates pool state

---

## CPI Implementation Example

Here's how the pool contract would make a CPI call to deploy:

```rust
use anchor_lang::prelude::*;
use ore_api;

pub fn mine(ctx: Context<Mine>) -> Result<()> {
    let pool = &mut ctx.accounts.pool;

    // Get current round from ORE board
    let board = &ctx.accounts.board;
    let round = &ctx.accounts.round;

    // Calculate bet size (e.g., 5% of pool balance per round)
    let bet_per_square = pool.total_sol_balance / 20 / pool.num_squares;

    // Select squares based on strategy
    let squares = match pool.strategy {
        PoolStrategy::TwentyFiveSquare => [true; 25],  // All squares
        PoolStrategy::EighteenSquare => select_18_least_crowded(round),
    };

    // Create deploy instruction
    let deploy_ix = ore_api::sdk::deploy(
        ctx.accounts.payer.key(),      // Signer (pays fees)
        pool.authority,                 // Pool's PDA (miner authority)
        bet_per_square,                 // Amount per square
        board.round_id,                 // Current round
        squares,                        // Which squares
    );

    // Get pool authority signer seeds
    let authority_seeds = &[
        b"pool_authority",
        &[pool.authority_bump],
    ];
    let signer_seeds = &[&authority_seeds[..]];

    // Make CPI call
    anchor_lang::solana_program::program::invoke_signed(
        &deploy_ix,
        &[
            ctx.accounts.payer.to_account_info(),
            ctx.accounts.pool_authority.to_account_info(),
            ctx.accounts.automation.to_account_info(),
            ctx.accounts.board.to_account_info(),
            ctx.accounts.miner.to_account_info(),
            ctx.accounts.round.to_account_info(),
            ctx.accounts.system_program.to_account_info(),
            ctx.accounts.entropy_var.to_account_info(),
            ctx.accounts.entropy_program.to_account_info(),
        ],
        signer_seeds,  // Pool authority signs
    )?;

    Ok(())
}
```

---

## Remaining Questions to Research

1. **Account Rent:** How much SOL is needed for rent-exempt accounts?
2. **Transaction Fees:** Who pays for CPI transaction fees? Pool or designated wallet?
3. **Automation Account:** What is the automation PDA and do we need to initialize it?
4. **Token Accounts:** How to properly set up ORE token accounts for pool?
5. **Testing:** Can we test this on devnet? Does devnet have active ORE v3?

---

## Next Steps

1. ✅ Understand ORE SDK capabilities
2. 🔄 Design proper pool architecture based on research
3. ⏳ Verify account rent requirements
4. ⏳ Build actual smart contract with proper CPI calls
5. ⏳ Test on devnet
6. ⏳ Security audit
7. ⏳ Mainnet deployment

---

## Conclusion

**It IS possible to build trustless ORE mining pools!**

Key requirements:
- Use pool PDA as miner authority
- Make CPI calls to ORE program for deploy/checkpoint/claim
- Maintain share-based accounting in pool contract
- Distribute rewards proportionally after claiming
- Have separate pools for different strategies

The previous "production ready" code was missing:
- Proper CPI implementation
- Correct account structure
- ORE program integration
- PDA signer seeds
- Proper instruction data encoding

Now we understand what's actually needed to build this properly.
