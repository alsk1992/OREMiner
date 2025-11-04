use anchor_lang::prelude::*;
use anchor_lang::solana_program::pubkey;

/// ORE program ID
pub const ORE_PROGRAM_ID: Pubkey = pubkey!("oreV3EG1i9BEgiAJ8b177Z2S2rMarzak4NMv1kULvWv");

/// ORE token mint address
pub const ORE_MINT: Pubkey = pubkey!("oreoU2P8bN6jkk3jbaiVxYnG1dCXcYxwhwyK9jSybcp");

/// ORE token decimals
pub const ORE_DECIMALS: u8 = 11;

/// One ORE token in smallest units
pub const ONE_ORE: u64 = 100_000_000_000; // 10^11

/// Minimum deposit amount (0.01 SOL)
pub const MIN_DEPOSIT: u64 = 10_000_000;

/// Maximum fee basis points (10% = 1000 bps)
pub const MAX_FEE_BPS: u16 = 1000;
