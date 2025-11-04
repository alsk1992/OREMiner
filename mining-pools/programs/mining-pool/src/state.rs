use anchor_lang::prelude::*;

/// Pool state account - one per strategy (25-square or 18-square)
#[account]
pub struct Pool {
    /// Pool authority PDA pubkey
    pub authority: Pubkey,

    /// PDA bump seed
    pub authority_bump: u8,

    /// Mining strategy (25 or 18 squares)
    pub strategy: PoolStrategy,

    /// Total shares issued to all users
    pub total_shares: u64,

    /// Total SOL deposited historically
    pub total_sol_deposited: u64,

    /// Current SOL balance available
    pub total_sol_current: u64,

    /// Total ORE claimed from mining (stored in pool's token account)
    pub total_ore_claimed: u64,

    /// Last round ID that was mined
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
        1 +  // strategy (enum)
        8 +  // total_shares
        8 +  // total_sol_deposited
        8 +  // total_sol_current
        8 +  // total_ore_claimed
        8 +  // last_round_id
        1 +  // paused
        32 + // fee_collector
        2;   // fee_basis_points
}

/// User deposit account - tracks individual user's shares
#[account]
pub struct UserDeposit {
    /// User's wallet pubkey
    pub user: Pubkey,

    /// Which pool this deposit belongs to
    pub pool: Pubkey,

    /// User's share amount
    pub shares: u64,

    /// How much SOL user deposited (for tracking only)
    pub deposited_sol: u64,

    /// Last round when user's rewards were calculated
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

/// Pool mining strategy
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub enum PoolStrategy {
    TwentyFiveSquare,  // Cover all 25 squares (100% win rate)
    EighteenSquare,    // Cover 18 least crowded (72%+ win rate)
}

impl PoolStrategy {
    /// Get strategy as bytes for PDA seeds
    pub fn to_bytes(&self) -> [u8; 1] {
        match self {
            PoolStrategy::TwentyFiveSquare => [0],
            PoolStrategy::EighteenSquare => [1],
        }
    }

    /// Get number of squares for this strategy
    pub fn num_squares(&self) -> usize {
        match self {
            PoolStrategy::TwentyFiveSquare => 25,
            PoolStrategy::EighteenSquare => 18,
        }
    }
}
