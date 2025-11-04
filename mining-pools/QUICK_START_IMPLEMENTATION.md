# Quick Start - Implementation Guide

When you're ready to start building, follow this guide.

---

## Prerequisites

```bash
# Install Anchor
cargo install --git https://github.com/coral-xyz/anchor avm --locked --force
avm install latest
avm use latest

# Verify Solana CLI
solana --version  # Should be 1.17+

# Verify Anchor
anchor --version  # Should be 0.29+
```

---

## Step 1: Project Setup (10 minutes)

```bash
cd /home/alsk/ore/mining-pools

# Initialize new Anchor project (or use existing)
# We already have the structure, just need to update it

# Update Cargo.toml
cat >> programs/mining-pool/Cargo.toml << 'EOF'

[dependencies]
anchor-lang = "0.29.0"
anchor-spl = "0.29.0"
ore-api = { git = "https://github.com/regolith-labs/ore", branch = "master" }
entropy-api = { git = "https://github.com/regolith-labs/entropy", branch = "master" }
solana-program = "1.17"
EOF

# Build to verify dependencies work
cd programs/mining-pool
cargo check
```

---

## Step 2: Define Constants (5 minutes)

**File:** `programs/mining-pool/src/lib.rs`

```rust
use anchor_lang::prelude::*;

declare_id!("YourProgramIDHere"); // Update after first build

pub mod state;
pub mod instructions;
pub mod error;

use instructions::*;

// ORE Program constants
pub const ORE_PROGRAM_ID: Pubkey = pubkey!("oreV3EG1i9BEgiAJ8b177Z2S2rMarzak4NMv1kULvWv");
pub const ORE_MINT: Pubkey = pubkey!("oreoU2P8bN6jkk3jbaiVxYnG1dCXcYxwhwyK9jSybcp");
pub const ONE_ORE: u64 = 100_000_000_000; // 11 decimals

#[program]
pub mod mining_pool {
    use super::*;

    pub fn initialize_pool(
        ctx: Context<InitializePool>,
        strategy: PoolStrategy,
        fee_basis_points: u16,
    ) -> Result<()> {
        instructions::initialize_pool::handler(ctx, strategy, fee_basis_points)
    }

    pub fn deposit(ctx: Context<Deposit>, amount: u64) -> Result<()> {
        instructions::deposit::handler(ctx, amount)
    }

    pub fn withdraw(ctx: Context<Withdraw>, shares: u64) -> Result<()> {
        instructions::withdraw::handler(ctx, shares)
    }

    pub fn mine(ctx: Context<Mine>) -> Result<()> {
        instructions::mine::handler(ctx)
    }

    pub fn checkpoint(ctx: Context<Checkpoint>, round_id: u64) -> Result<()> {
        instructions::checkpoint::handler(ctx, round_id)
    }

    pub fn claim_rewards(ctx: Context<ClaimRewards>) -> Result<()> {
        instructions::claim_rewards::handler(ctx)
    }

    pub fn pause(ctx: Context<Pause>) -> Result<()> {
        instructions::pause::handler(ctx)
    }
}
```

---

## Step 3: Define State Accounts (15 minutes)

**File:** `programs/mining-pool/src/state.rs`

```rust
use anchor_lang::prelude::*;

#[account]
pub struct Pool {
    /// Pool authority PDA
    pub authority: Pubkey,

    /// PDA bump seed
    pub authority_bump: u8,

    /// Mining strategy (25 or 18 squares)
    pub strategy: PoolStrategy,

    /// Total shares issued
    pub total_shares: u64,

    /// Total SOL deposited (historical)
    pub total_sol_deposited: u64,

    /// Current SOL balance
    pub total_sol_current: u64,

    /// Total ORE claimed from mining
    pub total_ore_claimed: u64,

    /// Last round mined
    pub last_round_id: u64,

    /// Emergency pause flag
    pub paused: bool,

    /// Where management fees go
    pub fee_collector: Pubkey,

    /// Management fee in basis points (200 = 2%)
    pub fee_basis_points: u16,
}

impl Pool {
    pub const LEN: usize = 8 + // discriminator
        32 + // authority
        1 +  // authority_bump
        1 +  // strategy
        8 +  // total_shares
        8 +  // total_sol_deposited
        8 +  // total_sol_current
        8 +  // total_ore_claimed
        8 +  // last_round_id
        1 +  // paused
        32 + // fee_collector
        2;   // fee_basis_points
}

#[account]
pub struct UserDeposit {
    /// User's wallet
    pub user: Pubkey,

    /// Which pool
    pub pool: Pubkey,

    /// User's share amount
    pub shares: u64,

    /// How much SOL deposited (for tracking only)
    pub deposited_sol: u64,

    /// Last round when rewards were calculated
    pub last_claim_round: u64,
}

impl UserDeposit {
    pub const LEN: usize = 8 + // discriminator
        32 + // user
        32 + // pool
        8 +  // shares
        8 +  // deposited_sol
        8;   // last_claim_round
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq)]
pub enum PoolStrategy {
    TwentyFiveSquare,  // 0
    EighteenSquare,    // 1
}

impl PoolStrategy {
    pub fn to_bytes(&self) -> [u8; 1] {
        match self {
            PoolStrategy::TwentyFiveSquare => [0],
            PoolStrategy::EighteenSquare => [1],
        }
    }

    pub fn num_squares(&self) -> usize {
        match self {
            PoolStrategy::TwentyFiveSquare => 25,
            PoolStrategy::EighteenSquare => 18,
        }
    }
}
```

---

## Step 4: Implement Initialize (20 minutes)

**File:** `programs/mining-pool/src/instructions/initialize_pool.rs`

```rust
use anchor_lang::prelude::*;
use crate::state::*;

#[derive(Accounts)]
#[instruction(strategy: PoolStrategy)]
pub struct InitializePool<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        init,
        payer = authority,
        space = Pool::LEN,
        seeds = [b"pool", strategy.to_bytes().as_ref()],
        bump
    )]
    pub pool: Account<'info, Pool>,

    /// Pool authority PDA (different from pool account!)
    #[account(
        seeds = [b"pool_authority", strategy.to_bytes().as_ref()],
        bump
    )]
    pub pool_authority: SystemAccount<'info>,

    /// Where management fees go
    /// CHECK: Can be any address
    pub fee_collector: AccountInfo<'info>,

    pub system_program: Program<'info, System>,
}

pub fn handler(
    ctx: Context<InitializePool>,
    strategy: PoolStrategy,
    fee_basis_points: u16,
) -> Result<()> {
    require!(fee_basis_points <= 1000, ErrorCode::FeeTooHigh); // Max 10%

    let pool = &mut ctx.accounts.pool;
    let authority_bump = ctx.bumps.pool_authority;

    pool.authority = ctx.accounts.pool_authority.key();
    pool.authority_bump = authority_bump;
    pool.strategy = strategy;
    pool.total_shares = 0;
    pool.total_sol_deposited = 0;
    pool.total_sol_current = 0;
    pool.total_ore_claimed = 0;
    pool.last_round_id = 0;
    pool.paused = false;
    pool.fee_collector = ctx.accounts.fee_collector.key();
    pool.fee_basis_points = fee_basis_points;

    msg!("Pool initialized: strategy={:?}, fee={}bp", strategy, fee_basis_points);

    Ok(())
}
```

---

## Step 5: Implement Deposit (20 minutes)

**File:** `programs/mining-pool/src/instructions/deposit.rs`

```rust
use anchor_lang::prelude::*;
use anchor_lang::system_program;
use crate::state::*;

#[derive(Accounts)]
pub struct Deposit<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(mut)]
    pub pool: Account<'info, Pool>,

    #[account(
        mut,
        seeds = [b"pool_authority", pool.strategy.to_bytes().as_ref()],
        bump = pool.authority_bump
    )]
    pub pool_authority: SystemAccount<'info>,

    #[account(
        init_if_needed,
        payer = user,
        space = UserDeposit::LEN,
        seeds = [b"user_deposit", pool.key().as_ref(), user.key().as_ref()],
        bump
    )]
    pub user_deposit: Account<'info, UserDeposit>,

    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<Deposit>, amount: u64) -> Result<()> {
    let pool = &mut ctx.accounts.pool;
    let user_deposit = &mut ctx.accounts.user_deposit;

    require!(!pool.paused, ErrorCode::PoolPaused);
    require!(amount >= 10_000_000, ErrorCode::DepositTooSmall); // Min 0.01 SOL

    // Transfer SOL from user to pool authority
    system_program::transfer(
        CpiContext::new(
            ctx.accounts.system_program.to_account_info(),
            system_program::Transfer {
                from: ctx.accounts.user.to_account_info(),
                to: ctx.accounts.pool_authority.to_account_info(),
            },
        ),
        amount,
    )?;

    // Calculate shares
    let shares_to_mint = if pool.total_shares == 0 {
        // First deposit: 1:1 ratio
        amount
    } else {
        // Subsequent deposits: proportional to current value
        (amount as u128)
            .checked_mul(pool.total_shares as u128)
            .unwrap()
            .checked_div(pool.total_sol_current as u128)
            .unwrap() as u64
    };

    // Update user deposit
    if user_deposit.user == Pubkey::default() {
        // First time initialization
        user_deposit.user = ctx.accounts.user.key();
        user_deposit.pool = pool.key();
        user_deposit.shares = shares_to_mint;
        user_deposit.deposited_sol = amount;
        user_deposit.last_claim_round = pool.last_round_id;
    } else {
        // Add to existing
        user_deposit.shares = user_deposit.shares.checked_add(shares_to_mint).unwrap();
        user_deposit.deposited_sol = user_deposit.deposited_sol.checked_add(amount).unwrap();
    }

    // Update pool
    pool.total_shares = pool.total_shares.checked_add(shares_to_mint).unwrap();
    pool.total_sol_deposited = pool.total_sol_deposited.checked_add(amount).unwrap();
    pool.total_sol_current = pool.total_sol_current.checked_add(amount).unwrap();

    msg!("Deposited {} lamports, minted {} shares", amount, shares_to_mint);

    Ok(())
}
```

---

## Step 6: Implement Withdraw (20 minutes)

**File:** `programs/mining-pool/src/instructions/withdraw.rs`

```rust
use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};
use crate::state::*;

#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(mut)]
    pub pool: Account<'info, Pool>,

    #[account(
        mut,
        seeds = [b"pool_authority", pool.strategy.to_bytes().as_ref()],
        bump = pool.authority_bump
    )]
    pub pool_authority: SystemAccount<'info>,

    #[account(
        mut,
        seeds = [b"user_deposit", pool.key().as_ref(), user.key().as_ref()],
        bump,
        constraint = user_deposit.user == user.key(),
    )]
    pub user_deposit: Account<'info, UserDeposit>,

    #[account(mut)]
    pub pool_ore_account: Account<'info, TokenAccount>,

    #[account(mut)]
    pub user_ore_account: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<Withdraw>, shares: u64) -> Result<()> {
    let pool = &mut ctx.accounts.pool;
    let user_deposit = &mut ctx.accounts.user_deposit;

    require!(shares > 0, ErrorCode::InvalidAmount);
    require!(user_deposit.shares >= shares, ErrorCode::InsufficientShares);
    require!(pool.total_shares > 0, ErrorCode::InvalidPoolState);

    // Calculate user's portion
    let sol_amount = (shares as u128)
        .checked_mul(pool.total_sol_current as u128)
        .unwrap()
        .checked_div(pool.total_shares as u128)
        .unwrap() as u64;

    let ore_amount = (shares as u128)
        .checked_mul(pool.total_ore_claimed as u128)
        .unwrap()
        .checked_div(pool.total_shares as u128)
        .unwrap() as u64;

    // Transfer SOL
    **ctx.accounts.pool_authority.to_account_info().try_borrow_mut_lamports()? -= sol_amount;
    **ctx.accounts.user.to_account_info().try_borrow_mut_lamports()? += sol_amount;

    // Transfer ORE (if any)
    if ore_amount > 0 {
        let authority_seeds = &[
            b"pool_authority",
            pool.strategy.to_bytes().as_ref(),
            &[pool.authority_bump],
        ];
        let signer_seeds = &[&authority_seeds[..]];

        token::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.pool_ore_account.to_account_info(),
                    to: ctx.accounts.user_ore_account.to_account_info(),
                    authority: ctx.accounts.pool_authority.to_account_info(),
                },
                signer_seeds,
            ),
            ore_amount,
        )?;
    }

    // Burn shares
    user_deposit.shares -= shares;
    pool.total_shares -= shares;
    pool.total_sol_current -= sol_amount;
    pool.total_ore_claimed -= ore_amount;

    msg!("Withdrew {} SOL, {} ORE for {} shares", sol_amount, ore_amount, shares);

    Ok(())
}
```

---

## Step 7: Test Basic Functionality (30 minutes)

**File:** `tests/basic.ts`

```typescript
import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { MiningPool } from "../target/types/mining_pool";
import { expect } from "chai";

describe("mining-pool-basic", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.MiningPool as Program<MiningPool>;
  const user = provider.wallet;

  it("Initializes pool", async () => {
    const strategy = { twentyFiveSquare: {} };
    const feeBps = 200; // 2%

    const [pool] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("pool"), Buffer.from([0])], // strategy = 0
      program.programId
    );

    const [poolAuthority] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("pool_authority"), Buffer.from([0])],
      program.programId
    );

    await program.methods
      .initializePool(strategy, feeBps)
      .accounts({
        authority: user.publicKey,
        pool,
        poolAuthority,
        feeCollector: user.publicKey,
      })
      .rpc();

    const poolAccount = await program.account.pool.fetch(pool);
    expect(poolAccount.strategy).to.deep.equal(strategy);
    expect(poolAccount.feeBasisPoints).to.equal(feeBps);
  });

  it("Deposits SOL", async () => {
    const amount = 1_000_000_000; // 1 SOL

    const [pool] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("pool"), Buffer.from([0])],
      program.programId
    );

    const [poolAuthority] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("pool_authority"), Buffer.from([0])],
      program.programId
    );

    const [userDeposit] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("user_deposit"), pool.toBuffer(), user.publicKey.toBuffer()],
      program.programId
    );

    await program.methods
      .deposit(new anchor.BN(amount))
      .accounts({
        user: user.publicKey,
        pool,
        poolAuthority,
        userDeposit,
      })
      .rpc();

    const depositAccount = await program.account.userDeposit.fetch(userDeposit);
    expect(depositAccount.shares.toString()).to.equal(amount.toString());
  });
});
```

**Run tests:**
```bash
anchor test
```

---

## Step 8: Implement Mine Instruction (1 hour)

This is where ORE CPI integration happens. See `IMPLEMENTATION_PLAN.md` section 2.3 for full code.

Key points:
- Add ore-api dependency
- Import ore_api functions
- Build deploy instruction
- Use invoke_signed with pool authority signer seeds
- Pass all required accounts

---

## Step 9: Test on Devnet (1 day)

```bash
# Check if ORE is on devnet
solana program show oreV3EG1i9BEgiAJ8b177Z2S2rMarzak4NMv1kULvWv --url devnet

# If yes:
anchor build
anchor deploy --provider.cluster devnet

# Initialize pools
# Deposit test amounts
# Run mining bot
# Verify everything works
```

---

## Common Errors and Solutions

### Error: "Account not initialized"
**Cause:** Trying to use account before it's created
**Fix:** Use `init_if_needed` or check account exists

### Error: "PDA does not match"
**Cause:** Wrong seeds or bump seed
**Fix:** Verify PDA derivation matches exactly

### Error: "Invalid signer seeds"
**Cause:** Wrong seeds in invoke_signed
**Fix:** Match seeds used in PDA derivation

### Error: "Insufficient lamports"
**Cause:** Account doesn't have enough SOL
**Fix:** Ensure proper rent and transfer amounts

---

## Debugging Tips

1. **Use msg!() liberally:**
```rust
msg!("Pool balance: {}", pool.total_sol_current);
msg!("Shares to mint: {}", shares);
```

2. **Check accounts in Solana Explorer:**
```bash
solana account <ADDRESS> --url devnet
```

3. **Run with verbose logs:**
```bash
anchor test -- --show-logs
```

4. **Use Anchor errors:**
```rust
#[error_code]
pub enum ErrorCode {
    #[msg("Pool is paused")]
    PoolPaused,
    #[msg("Insufficient shares")]
    InsufficientShares,
    // ...
}
```

---

## When You Get Stuck

1. Check `ORE_SDK_RESEARCH.md` for ORE-specific questions
2. Check `IMPLEMENTATION_PLAN.md` for detailed instruction code
3. Check `OPEN_QUESTIONS_RESOLVED.md` for common issues
4. Look at `/home/alsk/ore/api/src/` for ORE SDK examples
5. Look at `/home/alsk/ore/cli/src/deploy_optimal_ev.rs` for real mining code

---

## Success Criteria - Phase 1

Before moving to ORE integration:
- [ ] Pool can be initialized
- [ ] Users can deposit SOL
- [ ] Shares calculated correctly
- [ ] Users can withdraw proportionally
- [ ] Tests pass with 100% success rate

Once that works, add ORE integration!

---

**Good luck! You have everything you need to build this.** 🚀
