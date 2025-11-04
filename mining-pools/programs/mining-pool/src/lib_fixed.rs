use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Mint};
use ore_api::prelude::*;

declare_id!("11111111111111111111111111111111");

pub const ORE_PROGRAM_ID: Pubkey = solana_program::pubkey!("oreV3EG1i9BEgiAJ8b177Z2S2rMarzak4NMv1kULvWv");

#[program]
pub mod mining_pool {
    use super::*;

    pub fn initialize_pool(
        ctx: Context<InitializePool>,
        pool_type: PoolType,
        min_deposit: u64,
        management_fee_bps: u16,
    ) -> Result<()> {
        let pool = &mut ctx.accounts.pool;
        pool.authority = ctx.accounts.authority.key();
        pool.pool_type = pool_type;
        pool.total_deposited = 0;
        pool.total_shares = 0;
        pool.total_rounds = 0;
        pool.total_wins = 0;
        pool.total_ore_earned = 0;
        pool.total_sol_earned = 0;
        pool.min_deposit = min_deposit;
        pool.management_fee_bps = management_fee_bps;
        pool.is_active = true;
        pool.bump = ctx.bumps.pool;

        msg!("Pool initialized");
        Ok(())
    }

    pub fn deposit(ctx: Context<Deposit>, amount: u64) -> Result<()> {
        let pool = &mut ctx.accounts.pool;
        let depositor_account = &mut ctx.accounts.depositor_account;

        require!(pool.is_active, ErrorCode::PoolNotActive);
        require!(amount >= pool.min_deposit, ErrorCode::DepositTooSmall);

        anchor_lang::system_program::transfer(
            CpiContext::new(
                ctx.accounts.system_program.to_account_info(),
                anchor_lang::system_program::Transfer {
                    from: ctx.accounts.depositor.to_account_info(),
                    to: ctx.accounts.pool.to_account_info(),
                },
            ),
            amount,
        )?;

        let shares = if pool.total_shares == 0 {
            amount
        } else {
            let numerator = (amount as u128).checked_mul(pool.total_shares as u128).ok_or(ErrorCode::MathOverflow)?;
            let shares = numerator.checked_div(pool.total_deposited as u128).ok_or(ErrorCode::MathOverflow)?;
            require!(shares > 0, ErrorCode::SharesRoundedToZero);
            shares as u64
        };

        pool.total_deposited = pool.total_deposited.checked_add(amount).ok_or(ErrorCode::MathOverflow)?;
        pool.total_shares = pool.total_shares.checked_add(shares).ok_or(ErrorCode::MathOverflow)?;

        depositor_account.owner = ctx.accounts.depositor.key();
        depositor_account.shares = depositor_account.shares.checked_add(shares).ok_or(ErrorCode::MathOverflow)?;
        depositor_account.total_deposited = depositor_account.total_deposited.checked_add(amount).ok_or(ErrorCode::MathOverflow)?;

        msg!("Deposited {} lamports, received {} shares", amount, shares);
        Ok(())
    }

    pub fn withdraw(ctx: Context<Withdraw>, shares: u64) -> Result<()> {
        let pool = &mut ctx.accounts.pool;
        let depositor_account = &mut ctx.accounts.depositor_account;

        require!(depositor_account.shares >= shares, ErrorCode::InsufficientShares);
        require!(shares > 0, ErrorCode::InvalidAmount);

        let numerator = (shares as u128).checked_mul(pool.total_deposited as u128).ok_or(ErrorCode::MathOverflow)?;
        let amount = numerator.checked_div(pool.total_shares as u128).ok_or(ErrorCode::MathOverflow)? as u64;

        require!(amount > 0, ErrorCode::WithdrawTooSmall);

        let authority_seeds = &[b"pool".as_ref(), pool.authority.as_ref(), &[pool.bump]];
        let signer_seeds = &[&authority_seeds[..]];

        anchor_lang::system_program::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.system_program.to_account_info(),
                anchor_lang::system_program::Transfer {
                    from: ctx.accounts.pool.to_account_info(),
                    to: ctx.accounts.depositor.to_account_info(),
                },
                signer_seeds,
            ),
            amount,
        )?;

        pool.total_deposited = pool.total_deposited.checked_sub(amount).ok_or(ErrorCode::MathOverflow)?;
        pool.total_shares = pool.total_shares.checked_sub(shares).ok_or(ErrorCode::MathOverflow)?;
        depositor_account.shares = depositor_account.shares.checked_sub(shares).ok_or(ErrorCode::MathOverflow)?;

        msg!("Withdrew {} lamports", amount);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct InitializePool<'info> {
    #[account(init, payer = authority, space = 8 + Pool::SPACE, seeds = [b"pool", authority.key().as_ref()], bump)]
    pub pool: Account<'info, Pool>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Deposit<'info> {
    #[account(mut, seeds = [b"pool", pool.authority.as_ref()], bump = pool.bump)]
    pub pool: Account<'info, Pool>,
    #[account(init_if_needed, payer = depositor, space = 8 + DepositorAccount::SPACE, seeds = [b"depositor", pool.key().as_ref(), depositor.key().as_ref()], bump)]
    pub depositor_account: Account<'info, DepositorAccount>,
    #[account(mut)]
    pub depositor: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(mut, seeds = [b"pool", pool.authority.as_ref()], bump = pool.bump)]
    pub pool: Account<'info, Pool>,
    #[account(mut, seeds = [b"depositor", pool.key().as_ref(), depositor.key().as_ref()], bump)]
    pub depositor_account: Account<'info, DepositorAccount>,
    #[account(mut)]
    pub depositor: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[account]
pub struct Pool {
    pub authority: Pubkey,
    pub pool_type: PoolType,
    pub total_deposited: u64,
    pub total_shares: u64,
    pub total_rounds: u64,
    pub total_wins: u64,
    pub total_ore_earned: u64,
    pub total_sol_earned: u64,
    pub min_deposit: u64,
    pub management_fee_bps: u16,
    pub is_active: bool,
    pub bump: u8,
}

impl Pool {
    pub const SPACE: usize = 32 + 1 + 8 + 8 + 8 + 8 + 8 + 8 + 8 + 2 + 1 + 1;
}

#[account]
pub struct DepositorAccount {
    pub owner: Pubkey,
    pub shares: u64,
    pub total_deposited: u64,
}

impl DepositorAccount {
    pub const SPACE: usize = 32 + 8 + 8;
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq)]
pub enum PoolType {
    TwentyFiveSquare,
    EighteenSquare,
}

#[error_code]
pub enum ErrorCode {
    #[msg("Pool is not active")]
    PoolNotActive,
    #[msg("Deposit amount is below minimum")]
    DepositTooSmall,
    #[msg("Insufficient shares")]
    InsufficientShares,
    #[msg("Math overflow")]
    MathOverflow,
    #[msg("Shares would round to zero")]
    SharesRoundedToZero,
    #[msg("Invalid amount")]
    InvalidAmount,
    #[msg("Withdrawal amount too small")]
    WithdrawTooSmall,
}
