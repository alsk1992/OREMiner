use anchor_lang::prelude::*;
use crate::state::*;
use crate::error::*;
use crate::constants::*;

#[derive(Accounts)]
pub struct Checkpoint<'info> {
    /// Bot wallet that pays for transaction fees
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(mut)]
    pub pool: Account<'info, Pool>,

    /// Pool authority PDA
    /// CHECK: PDA verified by seeds
    #[account(
        seeds = [b"pool_authority", pool.strategy.to_bytes().as_ref()],
        bump = pool.authority_bump
    )]
    pub pool_authority: UncheckedAccount<'info>,

    // ORE PROGRAM ACCOUNTS

    /// Pool's ORE miner account
    /// CHECK: ORE program validates
    #[account(mut)]
    pub ore_miner: UncheckedAccount<'info>,

    /// ORE round account for the round being checkpointed
    /// CHECK: ORE program validates
    #[account(mut)]
    pub ore_round: UncheckedAccount<'info>,

    /// CHECK: ORE program ID
    #[account(constraint = ore_program.key() == ORE_PROGRAM_ID @ PoolError::OreCpiFailed)]
    pub ore_program: UncheckedAccount<'info>,

    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<Checkpoint>, round_id: u64) -> Result<()> {
    let pool = &ctx.accounts.pool;

    // Build ORE checkpoint instruction
    let checkpoint_ix = build_ore_checkpoint_instruction(
        ctx.accounts.pool_authority.key(),
        round_id,
    );

    // Prepare signer seeds
    let authority_seeds = &[
        b"pool_authority",
        pool.strategy.to_bytes().as_ref(),
        &[pool.authority_bump],
    ];
    let signer_seeds = &[&authority_seeds[..]];

    // Make CPI call to ORE program
    anchor_lang::solana_program::program::invoke_signed(
        &checkpoint_ix,
        &[
            ctx.accounts.pool_authority.to_account_info(),
            ctx.accounts.ore_miner.to_account_info(),
            ctx.accounts.ore_round.to_account_info(),
            ctx.accounts.system_program.to_account_info(),
        ],
        signer_seeds,
    )
    .map_err(|_| PoolError::OreCpiFailed)?;

    msg!("Checkpointed round #{} for pool", round_id);

    Ok(())
}

/// Helper: Build ORE checkpoint instruction
fn build_ore_checkpoint_instruction(
    authority: Pubkey,
    round_id: u64,
) -> anchor_lang::solana_program::instruction::Instruction {
    // Build instruction data (discriminator + Checkpoint struct)
    let data = vec![2u8]; // Checkpoint instruction discriminator

    // In production, use ore_api::sdk::checkpoint() which handles everything

    anchor_lang::solana_program::instruction::Instruction {
        program_id: ORE_PROGRAM_ID,
        accounts: vec![], // Would populate with actual accounts
        data,
    }
}
