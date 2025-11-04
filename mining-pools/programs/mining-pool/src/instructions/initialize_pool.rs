use anchor_lang::prelude::*;
use crate::state::*;
use crate::error::*;
use crate::constants::*;

#[derive(Accounts)]
#[instruction(strategy: PoolStrategy)]
pub struct InitializePool<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    /// Pool state account
    #[account(
        init,
        payer = authority,
        space = Pool::LEN,
        seeds = [b"pool", strategy.to_bytes().as_ref()],
        bump
    )]
    pub pool: Account<'info, Pool>,

    /// Pool authority PDA - this will act as the miner in ORE program
    /// CHECK: PDA verification handled by seeds constraint
    #[account(
        seeds = [b"pool_authority", strategy.to_bytes().as_ref()],
        bump
    )]
    pub pool_authority: UncheckedAccount<'info>,

    /// Where management fees will be sent
    /// CHECK: Can be any valid address
    pub fee_collector: UncheckedAccount<'info>,

    pub system_program: Program<'info, System>,
}

pub fn handler(
    ctx: Context<InitializePool>,
    strategy: PoolStrategy,
    fee_basis_points: u16,
) -> Result<()> {
    require!(
        fee_basis_points <= MAX_FEE_BPS,
        PoolError::FeeTooHigh
    );

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

    msg!(
        "Pool initialized: strategy={:?}, fee={}bps",
        strategy,
        fee_basis_points
    );

    Ok(())
}
