//! Instruction handlers for the claim resolver.

pub mod claim;
pub mod dispute;
pub mod settle;

pub use claim::*;
pub use dispute::*;
pub use settle::*;

/// PDA seed for the singleton governance config.
pub const GOVERNANCE_SEED: &[u8] = b"governance";
/// PDA seed for a claim.
pub const CLAIM_SEED: &[u8] = b"claim";
/// PDA seed for a dispute.
pub const DISPUTE_SEED: &[u8] = b"dispute";
/// PDA seed for a vote receipt.
pub const VOTE_SEED: &[u8] = b"vote";
