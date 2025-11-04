// Mining Pool Contract using Steel Framework
// Based on ORE program architecture

mod initialize;
mod deposit;
mod withdraw;
mod mine;

use initialize::*;
use deposit::*;
use withdraw::*;
use mine::*;

use ore_api::prelude::*;
use steel::*;

// Pool instruction discriminators
pub const INITIALIZE_POOL: u8 = 0;
pub const DEPOSIT: u8 = 1;
pub const WITHDRAW: u8 = 2;
pub const MINE: u8 = 3;
pub const CHECKPOINT: u8 = 4;

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    data: &[u8],
) -> ProgramResult {
    // Parse instruction discriminator
    let (discriminator, data) = data.split_first().ok_or(ProgramError::InvalidInstructionData)?;

    match *discriminator {
        INITIALIZE_POOL => process_initialize(accounts, data)?,
        DEPOSIT => process_deposit(accounts, data)?,
        WITHDRAW => process_withdraw(accounts, data)?,
        MINE => process_mine(accounts, data)?,
        CHECKPOINT => process_checkpoint(accounts, data)?,
        _ => return Err(ProgramError::InvalidInstructionData),
    }

    Ok(())
}

entrypoint!(process_instruction);

// Pool program ID - will be set after deployment
declare_id!("11111111111111111111111111111111");

// PDA seeds
pub const POOL: &[u8] = b"pool";
pub const POOL_AUTHORITY: &[u8] = b"pool_authority";
pub const USER_DEPOSIT: &[u8] = b"user_deposit";

// Constants
pub const MIN_DEPOSIT: u64 = 10_000_000; // 0.01 SOL
pub const MAX_FEE_BPS: u16 = 1000; // 10%
