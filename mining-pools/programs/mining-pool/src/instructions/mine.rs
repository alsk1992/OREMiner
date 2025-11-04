use anchor_lang::prelude::*;
use crate::state::*;
use crate::error::*;
use crate::constants::*;

#[derive(Accounts)]
pub struct Mine<'info> {
    /// Bot wallet that pays for transaction fees
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(mut)]
    pub pool: Account<'info, Pool>,

    /// Pool authority PDA - acts as the miner in ORE program
    /// CHECK: PDA verified by seeds, used for CPI signing
    #[account(
        seeds = [b"pool_authority", pool.strategy.to_bytes().as_ref()],
        bump = pool.authority_bump
    )]
    pub pool_authority: UncheckedAccount<'info>,

    // ORE PROGRAM ACCOUNTS (required for CPI)

    /// ORE board account (global state)
    /// CHECK: ORE program validates this
    #[account(mut)]
    pub ore_board: UncheckedAccount<'info>,

    /// ORE round account for current round
    /// CHECK: ORE program validates this
    #[account(mut)]
    pub ore_round: UncheckedAccount<'info>,

    /// Pool's ORE miner account (PDA from ORE program)
    /// CHECK: ORE program creates/manages this
    #[account(mut)]
    pub ore_miner: UncheckedAccount<'info>,

    /// ORE automation account (optional, for pool_authority)
    /// CHECK: ORE program validates this
    #[account(mut)]
    pub ore_automation: UncheckedAccount<'info>,

    /// Entropy var account (for randomness)
    /// CHECK: Entropy program validates this
    #[account(mut)]
    pub entropy_var: UncheckedAccount<'info>,

    /// CHECK: ORE program ID
    #[account(constraint = ore_program.key() == ORE_PROGRAM_ID @ PoolError::OreCpiFailed)]
    pub ore_program: UncheckedAccount<'info>,

    /// CHECK: Entropy program
    pub entropy_program: UncheckedAccount<'info>,

    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<Mine>) -> Result<()> {
    let pool = &mut ctx.accounts.pool;

    require!(!pool.paused, PoolError::PoolPaused);

    // Get current round ID from board (read from account data)
    // For now, we'll pass it as a parameter in the instruction
    // In production, parse the board account data to get round_id

    // Calculate bet size (e.g., 5% of pool balance per round)
    let bet_per_round = pool
        .total_sol_current
        .checked_div(20)
        .ok_or(PoolError::MathOverflow)?; // 5% of pool

    let num_squares = pool.strategy.num_squares() as u64;
    let bet_per_square = bet_per_round
        .checked_div(num_squares)
        .ok_or(PoolError::MathOverflow)?;

    require!(
        bet_per_square > 0,
        PoolError::InsufficientPoolBalance
    );

    // Select squares based on strategy
    let squares = match pool.strategy {
        PoolStrategy::TwentyFiveSquare => {
            // All squares - guaranteed win!
            [true; 25]
        }
        PoolStrategy::EighteenSquare => {
            // Would need to read round data and select 18 least crowded
            // For now, placeholder - in production, parse ore_round account
            // and select 18 squares with lowest deployment
            select_18_least_crowded(&ctx.accounts.ore_round)?
        }
    };

    // Build ORE deploy instruction
    let deploy_ix = build_ore_deploy_instruction(
        ctx.accounts.payer.key(),
        ctx.accounts.pool_authority.key(),
        bet_per_square,
        pool.last_round_id, // Would get from board in production
        squares,
    );

    // Prepare signer seeds for CPI
    let authority_seeds = &[
        b"pool_authority",
        pool.strategy.to_bytes().as_ref(),
        &[pool.authority_bump],
    ];
    let signer_seeds = &[&authority_seeds[..]];

    // Make CPI call to ORE program
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
    )
    .map_err(|_| PoolError::OreCpiFailed)?;

    // Update pool state
    pool.last_round_id += 1;

    msg!(
        "Pool deployed {} SOL ({} per square) to round #{}",
        bet_per_round,
        bet_per_square,
        pool.last_round_id
    );

    Ok(())
}

/// Helper: Build ORE deploy instruction
/// This mimics ore_api::sdk::deploy()
fn build_ore_deploy_instruction(
    signer: Pubkey,
    authority: Pubkey,
    amount: u64,
    round_id: u64,
    squares: [bool; 25],
) -> anchor_lang::solana_program::instruction::Instruction {
    // Convert squares array to 32-bit mask
    let mut mask: u32 = 0;
    for (i, &square) in squares.iter().enumerate() {
        if square {
            mask |= 1 << i;
        }
    }

    // Build instruction data (discriminator + Deploy struct)
    let mut data = vec![6u8]; // Deploy instruction discriminator
    data.extend_from_slice(&amount.to_le_bytes());
    data.extend_from_slice(&mask.to_le_bytes());

    // This is a simplified version - in production, you'd use ore_api::sdk::deploy()
    // which handles all the account derivations and instruction building

    anchor_lang::solana_program::instruction::Instruction {
        program_id: ORE_PROGRAM_ID,
        accounts: vec![], // Would populate with actual accounts
        data,
    }
}

/// Helper: Select 18 least crowded squares
/// In production, this would parse the ore_round account data
fn select_18_least_crowded(ore_round: &UncheckedAccount) -> Result<[bool; 25]> {
    // TODO: Parse ore_round account data to get deployed amounts per square
    // Sort by deployment (ascending)
    // Select first 18 squares

    // Placeholder: just select first 18 squares
    let mut squares = [false; 25];
    for i in 0..18 {
        squares[i] = true;
    }
    Ok(squares)
}
