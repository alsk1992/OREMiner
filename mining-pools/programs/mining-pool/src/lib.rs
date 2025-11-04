use anchor_lang::prelude::*;

declare_id!("11111111111111111111111111111111");

pub mod state;
pub mod instructions;
pub mod error;
pub mod constants;

use instructions::*;

#[program]
pub mod mining_pool {
    use super::*;

    pub fn initialize_pool(
        ctx: Context<InitializePool>,
        strategy: state::PoolStrategy,
        fee_basis_points: u16,
    ) -> Result<()> {
        instructions::initialize_pool::handler(ctx, strategy, fee_basis_points)
    }

    pub fn deposit(ctx: Context<Deposit>, amount: u64) -> Result<()> {
        instructions::deposit::handler(ctx, amount)
    }

    pub fn withdraw(ctx: Context<Withdraw>, shares: u64, claim_ore: bool) -> Result<()> {
        instructions::withdraw::handler(ctx, shares, claim_ore)
    }

    pub fn mine(ctx: Context<Mine>) -> Result<()> {
        instructions::mine::handler(ctx)
    }

    pub fn checkpoint(ctx: Context<Checkpoint>, round_id: u64) -> Result<()> {
        instructions::checkpoint::handler(ctx, round_id)
    }
}
