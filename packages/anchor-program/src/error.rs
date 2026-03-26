//! Error codes for the cover executor.

use anchor_lang::prelude::*;

/// Errors returned by the berm-cover program.
#[error_code]
pub enum CoverError {
    /// Coverage or premium amount was zero.
    #[msg("amount must be greater than zero")]
    ZeroAmount,
    /// Requested duration is outside the product's allowed range.
    #[msg("cover duration out of allowed range")]
    DurationOutOfRange,
    /// Arithmetic overflow.
    #[msg("arithmetic overflow")]
    Overflow,
    /// The cover product is not accepting new policies.
    #[msg("cover product is inactive")]
    ProductInactive,
    /// Coverage exceeds the product's remaining capacity.
    #[msg("coverage exceeds product capacity")]
    CapacityExceeded,
    /// Underwriter stake is below the protocol minimum.
    #[msg("underwriter stake below minimum")]
    StakeTooLow,
    /// Caller is not the expected authority.
    #[msg("unauthorized")]
    Unauthorized,
    /// Oracle account could not be parsed.
    #[msg("failed to parse oracle account")]
    OracleParse,
    /// Oracle result is stale.
    #[msg("oracle result is stale")]
    StaleOracle,
    /// Provided risk score is out of the valid 0..=1000 range.
    #[msg("risk score out of range")]
    RiskScoreOutOfRange,
}
