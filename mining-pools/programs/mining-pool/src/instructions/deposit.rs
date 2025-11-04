use anchor_lang::prelude::*;
use anchor_lang::system_program;
use crate::state::*;
use crate::error::*;
use crate::constants::*;

#[derive(Accounts)]
pub struct Deposit<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(mut)]
    pub pool: Account<'info, Pool>,

    /// Pool authority PDA - receives the deposited SOL
    /// CHECK: PDA verified by seeds
    #[account(
        mut,
        seeds = [b"pool_authority", pool.strategy.to_bytes().as_ref()],
        bump = pool.authority_bump
    )]
    pub pool_authority: UncheckedAccount<'info>,

    /// User's deposit tracking account
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

    require!(!pool.paused, PoolError::PoolPaused);
    require!(amount >= MIN_DEPOSIT, PoolError::DepositTooSmall);

    // Transfer SOL from user to pool authority PDA
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

    // Calculate shares to mint
    let shares_to_mint = if pool.total_shares == 0 {
        // First deposit: 1:1 ratio
        amount
    } else {
        // Subsequent deposits: proportional to current pool value
        amount
            .checked_mul(pool.total_shares)
            .ok_or(PoolError::MathOverflow)?
            .checked_div(pool.total_sol_current)
            .ok_or(PoolError::MathOverflow)?
    };

    // Initialize or update user deposit account
    if user_deposit.user == Pubkey::default() {
        // First time setup
        user_deposit.user = ctx.accounts.user.key();
        user_deposit.pool = pool.key();
        user_deposit.shares = shares_to_mint;
        user_deposit.deposited_sol = amount;
        user_deposit.last_claim_round = pool.last_round_id;
    } else {
        // Add to existing
        user_deposit.shares = user_deposit
            .shares
            .checked_add(shares_to_mint)
            .ok_or(PoolError::MathOverflow)?;
        user_deposit.deposited_sol = user_deposit
            .deposited_sol
            .checked_add(amount)
            .ok_or(PoolError::MathOverflow)?;
    }

    // Update pool totals
    pool.total_shares = pool
        .total_shares
        .checked_add(shares_to_mint)
        .ok_or(PoolError::MathOverflow)?;
    pool.total_sol_deposited = pool
        .total_sol_deposited
        .checked_add(amount)
        .ok_or(PoolError::MathOverflow)?;
    pool.total_sol_current = pool
        .total_sol_current
        .checked_add(amount)
        .ok_or(PoolError::MathOverflow)?;

    msg!(
        "User deposited {} lamports, received {} shares",
        amount,
        shares_to_mint
    );

    Ok(())
}
