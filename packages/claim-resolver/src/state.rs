//! On-chain account state for the claim resolver.

use anchor_lang::prelude::*;

/// Governance parameters for dispute resolution.
#[account]
#[derive(InitSpace)]
pub struct GovernanceConfig {
    /// Authority allowed to update governance parameters.
    pub authority: Pubkey,
    /// Registered keeper authorised to submit off-chain attestations.
    pub keeper: Pubkey,
    /// Total eligible voting weight (staked $BERM snapshot).
    pub eligible_weight: u64,
    /// Quorum requirement (bps of eligible weight).
    pub quorum_bps: u64,
    /// Approval threshold (bps of cast weight).
    pub approval_threshold_bps: u64,
    /// Voting window length in slots.
    pub voting_slots: u64,
    /// PDA bump.
    pub bump: u8,
}

/// Lifecycle status of a claim.
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq, Debug, InitSpace)]
pub enum ClaimStatus {
    /// Opened, awaiting resolution.
    Pending,
    /// Parametric condition confirmed; ready for payout.
    Approved,
    /// Escalated to a governance dispute.
    Disputed,
    /// Paid out.
    Settled,
    /// Rejected (condition not met or vote failed).
    Rejected,
}

/// A claim against a policy.
#[account]
#[derive(InitSpace)]
pub struct Claim {
    /// The policy this claim references.
    pub policy: Pubkey,
    /// The claimant who opened it.
    pub claimant: Pubkey,
    /// Cover-type ordinal (matches the engine / cover executor).
    pub cover_type: u8,
    /// Coverage limit copied from the policy at open time.
    pub coverage: u64,
    /// Confirmed payout amount (set on approval).
    pub payout: u64,
    /// Trigger depth in bps that justified the payout.
    pub trigger_bps: u64,
    /// Current status.
    pub status: ClaimStatus,
    /// Slot the claim was opened at.
    pub opened_slot: u64,
    /// PDA bump.
    pub bump: u8,
}

/// A dispute escalated from a contested claim.
#[account]
#[derive(InitSpace)]
pub struct Dispute {
    /// The claim under dispute.
    pub claim: Pubkey,
    /// Accumulated approve weight.
    pub approve_weight: u64,
    /// Accumulated reject weight.
    pub reject_weight: u64,
    /// Slot voting opened.
    pub opened_slot: u64,
    /// Slot voting closes.
    pub closes_slot: u64,
    /// Whether the dispute has been finalised.
    pub finalized: bool,
    /// PDA bump.
    pub bump: u8,
}

/// A single voter's ballot on a dispute (prevents double voting).
#[account]
#[derive(InitSpace)]
pub struct VoteReceipt {
    /// Dispute voted on.
    pub dispute: Pubkey,
    /// Voter.
    pub voter: Pubkey,
    /// Weight cast.
    pub weight: u64,
    /// Whether the vote was in favour.
    pub approve: bool,
    /// PDA bump.
    pub bump: u8,
}
