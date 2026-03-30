//! On-chain account state for the cover executor.

use anchor_lang::prelude::*;

/// Global protocol configuration (one per deployment).
#[account]
#[derive(InitSpace)]
pub struct Protocol {
    /// Governance authority.
    pub authority: Pubkey,
    /// Treasury that receives the protocol fee share of premiums.
    pub treasury: Pubkey,
    /// Protocol fee on premiums, in basis points.
    pub fee_bps: u16,
    /// Count of cover products created.
    pub total_products: u64,
    /// Count of policies issued.
    pub total_policies: u64,
    /// Global pause switch.
    pub paused: bool,
    /// PDA bump.
    pub bump: u8,
}

/// A cover product: a purchasable cover of one type on one subject.
#[account]
#[derive(InitSpace)]
pub struct CoverProduct {
    /// Owning protocol.
    pub protocol: Pubkey,
    /// Cover-type ordinal (see [`crate::cover_type::CoverType`]).
    pub cover_type: u8,
    /// Subject identifier (protocol name / asset symbol / validator id).
    #[max_len(48)]
    pub subject: String,
    /// Category base rate (bps) before risk adjustment.
    pub base_rate_bps: u64,
    /// Latest underwriter safety score for the subject (0..=1000).
    pub safety_score: u64,
    /// Maximum aggregate coverage this product can sell (asset units).
    pub capacity: u64,
    /// Coverage currently committed across active policies.
    pub committed: u64,
    /// Minimum policy duration (days).
    pub min_duration_days: u32,
    /// Maximum policy duration (days).
    pub max_duration_days: u32,
    /// Whether the product accepts new policies.
    pub active: bool,
    /// PDA bump.
    pub bump: u8,
}

impl CoverProduct {
    /// Remaining sellable capacity.
    pub fn remaining_capacity(&self) -> u64 {
        self.capacity.saturating_sub(self.committed)
    }
}

/// Status of an issued policy.
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq, Debug, InitSpace)]
pub enum PolicyStatus {
    /// In force.
    Active,
    /// Triggered and settled by the claim resolver.
    Settled,
    /// Expired without triggering.
    Expired,
}

/// An issued cover policy.
#[account]
#[derive(InitSpace)]
pub struct CoverPolicy {
    /// The product this policy was bought from.
    pub product: Pubkey,
    /// Protected wallet.
    pub holder: Pubkey,
    /// Coverage limit (asset units).
    pub coverage: u64,
    /// Premium paid (asset units).
    pub premium_paid: u64,
    /// Unix timestamp cover begins.
    pub start_ts: i64,
    /// Unix timestamp cover ends.
    pub end_ts: i64,
    /// Lifecycle status.
    pub status: PolicyStatus,
    /// Monotonic policy index within the protocol.
    pub index: u64,
    /// PDA bump.
    pub bump: u8,
}

/// A registered underwriter who stakes $BERM and assesses protocol risk.
#[account]
#[derive(InitSpace)]
pub struct Underwriter {
    /// Underwriter authority.
    pub authority: Pubkey,
    /// Staked amount backing the underwriter's assessments.
    pub stake: u64,
    /// Reputation score accumulated from correct assessments.
    pub reputation: u64,
    /// Number of products this underwriter has scored.
    pub products_assessed: u64,
    /// Whether the underwriter is currently slashed/disabled.
    pub slashed: bool,
    /// PDA bump.
    pub bump: u8,
}
