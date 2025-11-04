use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};
use crate::state::*;
use crate::error::*;

#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(mut)]
    pub pool: Account<'info, Pool>,

    /// Pool authority PDA - holds the SOL
    /// CHECK: PDA verified by seeds
    #[account(
        mut,
        seeds = [b"pool_authority", pool.strategy.to_bytes().as_ref()],
        bump = pool.authority_bump
    )]
    pub pool_authority: UncheckedAccount<'info>,

    #[account(
        mut,
        seeds = [b"user_deposit", pool.key().as_ref(), user.key().as_ref()],
        bump,
        constraint = user_deposit.user == user.key() @ PoolError::Unauthorized,
    )]
    pub user_deposit: Account<'info, UserDeposit>,

    /// Pool's ORE token account (only if claiming ORE)
    #[account(mut)]
    pub pool_ore_account: Option<Account<'info, TokenAccount>>,

    /// User's ORE token account (only if claiming ORE)
    #[account(mut)]
    pub user_ore_account: Option<Account<'info, TokenAccount>>,

    pub token_program: Option<Program<'info, Token>>,
    pub system_program: Program<'info, System>,
}

pub fn handler(
    ctx: Context<Withdraw>,
    shares: u64,
    claim_ore: bool, // KEY: User chooses whether to claim ORE or leave it staking @ 150% APR!
) -> Result<()> {
    let pool = &mut ctx.accounts.pool;
    let user_deposit = &mut ctx.accounts.user_deposit;

    require!(shares > 0, PoolError::InvalidPoolState);
    require!(
        user_deposit.shares >= shares,
        PoolError::InsufficientShares
    );
    require!(pool.total_shares > 0, PoolError::InvalidPoolState);

    // Calculate user's portion of SOL
    let sol_amount = (shares as u128)
        .checked_mul(pool.total_sol_current as u128)
        .ok_or(PoolError::MathOverflow)?
        .checked_div(pool.total_shares as u128)
        .ok_or(PoolError::MathOverflow)?
        as u64;

    // Transfer SOL from pool authority to user
    **ctx
        .accounts
        .pool_authority
        .to_account_info()
        .try_borrow_mut_lamports()? -= sol_amount;
    **ctx
        .accounts
        .user
        .to_account_info()
        .try_borrow_mut_lamports()? += sol_amount;

    // Transfer ORE ONLY IF USER WANTS IT (otherwise leave staking @ 150% APR!)
    let mut ore_amount = 0u64;
    if claim_ore {
        if let (Some(pool_ore_account), Some(user_ore_account), Some(token_program)) = (
            &ctx.accounts.pool_ore_account,
            &ctx.accounts.user_ore_account,
            &ctx.accounts.token_program,
        ) {
            // Calculate user's portion of ORE
            ore_amount = (shares as u128)
                .checked_mul(pool.total_ore_claimed as u128)
                .ok_or(PoolError::MathOverflow)?
                .checked_div(pool.total_shares as u128)
                .ok_or(PoolError::MathOverflow)?
                as u64;

            if ore_amount > 0 {
                // Transfer ORE with pool authority as signer
                let authority_seeds = &[
                    b"pool_authority",
                    pool.strategy.to_bytes().as_ref(),
                    &[pool.authority_bump],
                ];
                let signer_seeds = &[&authority_seeds[..]];

                token::transfer(
                    CpiContext::new_with_signer(
                        token_program.to_account_info(),
                        Transfer {
                            from: pool_ore_account.to_account_info(),
                            to: user_ore_account.to_account_info(),
                            authority: ctx.accounts.pool_authority.to_account_info(),
                        },
                        signer_seeds,
                    ),
                    ore_amount,
                )?;

                // Update pool ORE balance
                pool.total_ore_claimed = pool
                    .total_ore_claimed
                    .checked_sub(ore_amount)
                    .ok_or(PoolError::MathOverflow)?;
            }
        }
    }

    // Burn user's shares
    user_deposit.shares = user_deposit
        .shares
        .checked_sub(shares)
        .ok_or(PoolError::MathOverflow)?;

    // Update pool totals
    pool.total_shares = pool
        .total_shares
        .checked_sub(shares)
        .ok_or(PoolError::MathOverflow)?;
    pool.total_sol_current = pool
        .total_sol_current
        .checked_sub(sol_amount)
        .ok_or(PoolError::MathOverflow)?;

    msg!(
        "User withdrew {} SOL, {} ORE (claimed: {}) for {} shares",
        sol_amount,
        ore_amount,
        claim_ore,
        shares
    );

    Ok(())
}
