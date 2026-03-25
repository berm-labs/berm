//! Events emitted by the claim resolver.

use anchor_lang::prelude::*;

/// Emitted when a claim is opened.
#[event]
pub struct ClaimOpened {
    /// Claim account.
    pub claim: Pubkey,
    /// Policy referenced.
    pub policy: Pubkey,
    /// Cover-type ordinal.
    pub cover_type: u8,
}

/// Emitted when a claim auto-resolves on a parametric trigger.
#[event]
pub struct ClaimAutoResolved {
    /// Claim account.
    pub claim: Pubkey,
    /// Confirmed payout.
    pub payout: u64,
    /// Trigger depth in bps.
    pub trigger_bps: u64,
}

/// Emitted when a dispute is opened.
#[event]
pub struct DisputeOpened {
    /// Dispute account.
    pub dispute: Pubkey,
    /// Claim under dispute.
    pub claim: Pubkey,
    /// Slot voting closes.
    pub closes_slot: u64,
}

/// Emitted when a dispute is finalised.
#[event]
pub struct DisputeFinalized {
    /// Dispute account.
    pub dispute: Pubkey,
    /// Whether the claim was approved.
    pub approved: bool,
    /// Final approve weight.
    pub approve_weight: u64,
    /// Final reject weight.
    pub reject_weight: u64,
}

/// Emitted when a claim is settled (paid out).
#[event]
pub struct ClaimSettled {
    /// Claim account.
    pub claim: Pubkey,
    /// Amount paid.
    pub payout: u64,
}
