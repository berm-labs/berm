//! Vault error codes.

use anchor_lang::prelude::*;

/// Errors returned by the pool-vault program.
#[error_code]
pub enum VaultError {
    /// Deposit or withdraw amount was zero.
    #[msg("amount must be greater than zero")]
    ZeroAmount,
    /// Share/asset conversion overflowed.
    #[msg("arithmetic overflow")]
    Overflow,
    /// Withdrawal would exceed the depositor's share balance.
    #[msg("insufficient shares for withdrawal")]
    InsufficientShares,
    /// Withdrawal would drop the pool below its locked-cover backing.
    #[msg("withdrawal would breach the pool solvency floor")]
    SolvencyFloorBreached,
    /// The pool is paused for deposits/withdrawals.
    #[msg("pool is paused")]
    PoolPaused,
    /// Caller is not the pool authority.
    #[msg("unauthorized")]
    Unauthorized,
    /// First deposit was below the minimum-liquidity bootstrap.
    #[msg("first deposit below minimum liquidity")]
    BelowMinimumLiquidity,
}
