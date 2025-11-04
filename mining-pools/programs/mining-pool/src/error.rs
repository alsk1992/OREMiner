use anchor_lang::prelude::*;

#[error_code]
pub enum PoolError {
    #[msg("Pool is paused")]
    PoolPaused,

    #[msg("Deposit amount is below minimum")]
    DepositTooSmall,

    #[msg("Insufficient shares to withdraw")]
    InsufficientShares,

    #[msg("Management fee exceeds maximum (10%)")]
    FeeTooHigh,

    #[msg("Invalid pool state")]
    InvalidPoolState,

    #[msg("Pool has insufficient balance")]
    InsufficientPoolBalance,

    #[msg("Unauthorized - only pool authority can call this")]
    Unauthorized,

    #[msg("Invalid square selection for strategy")]
    InvalidSquareSelection,

    #[msg("ORE CPI call failed")]
    OreCpiFailed,

    #[msg("Math overflow")]
    MathOverflow,
}
