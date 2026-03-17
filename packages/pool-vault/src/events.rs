//! Events emitted by the vault for off-chain indexers (the dashboard, the SDK).

use anchor_lang::prelude::*;

/// Emitted when a cover pool is created.
#[event]
pub struct PoolInitialized {
    /// The new pool account.
    pub pool: Pubkey,
    /// Cover type ordinal.
    pub cover_type: u8,
    /// Asset mint.
    pub asset_mint: Pubkey,
}

/// Emitted on every LP deposit.
#[event]
pub struct Deposited {
    /// Pool deposited into.
    pub pool: Pubkey,
    /// Depositor.
    pub owner: Pubkey,
    /// Assets deposited.
    pub assets: u64,
    /// Shares minted.
    pub shares: u64,
}

/// Emitted on every LP withdrawal.
#[event]
pub struct Withdrawn {
    /// Pool withdrawn from.
    pub pool: Pubkey,
    /// Withdrawer.
    pub owner: Pubkey,
    /// Shares burned.
    pub shares: u64,
    /// Assets returned.
    pub assets: u64,
}

/// Emitted when premium revenue is distributed into a pool.
#[event]
pub struct PremiumDistributed {
    /// Pool that received premium.
    pub pool: Pubkey,
    /// Premium amount added to AUM.
    pub amount: u64,
    /// New total assets.
    pub total_assets: u64,
}
