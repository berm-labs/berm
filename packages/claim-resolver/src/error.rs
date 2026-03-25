//! Claim-resolver error codes.

use anchor_lang::prelude::*;

/// Errors returned by the claim-resolver program.
#[error_code]
pub enum ResolverError {
    /// The claim is not in a state that allows this transition.
    #[msg("invalid claim state for this action")]
    InvalidState,
    /// The parametric condition was not met, so the claim cannot auto-resolve.
    #[msg("parametric trigger condition not met")]
    TriggerNotMet,
    /// Caller is not an authorised keeper.
    #[msg("unauthorized keeper")]
    UnauthorizedKeeper,
    /// Caller is not authorised for this action.
    #[msg("unauthorized")]
    Unauthorized,
    /// Voting is still open; the dispute cannot be finalised yet.
    #[msg("voting window still open")]
    VotingOpen,
    /// Quorum was not reached.
    #[msg("quorum not reached")]
    QuorumNotReached,
    /// The voter already cast a ballot on this dispute.
    #[msg("already voted")]
    AlreadyVoted,
    /// Oracle account could not be parsed or is stale.
    #[msg("oracle read failed")]
    OracleRead,
    /// Arithmetic overflow.
    #[msg("arithmetic overflow")]
    Overflow,
    /// Payout exceeds the policy coverage.
    #[msg("payout exceeds coverage")]
    PayoutExceedsCoverage,
}
