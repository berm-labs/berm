//! Events emitted by the cover executor for off-chain indexers.

use anchor_lang::prelude::*;

/// Emitted when the protocol is initialised.
#[event]
pub struct ProtocolInitialized {
    /// Protocol account.
    pub protocol: Pubkey,
    /// Governance authority.
    pub authority: Pubkey,
}

/// Emitted when a cover product is created.
#[event]
pub struct ProductCreated {
    /// Product account.
    pub product: Pubkey,
    /// Cover-type ordinal.
    pub cover_type: u8,
    /// Subject identifier.
    pub subject: String,
    /// Sellable capacity.
    pub capacity: u64,
}

/// Emitted when a policy is purchased.
#[event]
pub struct CoverPurchased {
    /// Policy account.
    pub policy: Pubkey,
    /// Product the policy was bought from.
    pub product: Pubkey,
    /// Protected wallet.
    pub holder: Pubkey,
    /// Coverage limit.
    pub coverage: u64,
    /// Premium paid.
    pub premium: u64,
}

/// Emitted when an underwriter (re)scores a product.
#[event]
pub struct RiskAssessed {
    /// Product scored.
    pub product: Pubkey,
    /// Underwriter that scored it.
    pub underwriter: Pubkey,
    /// New safety score (0..=1000).
    pub safety_score: u64,
}
