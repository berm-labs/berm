//! Program-wide constants and PDA seeds for the cover-pool vault.

/// Basis-point denominator.
pub const BPS: u64 = 10_000;

/// PDA seed for a [`crate::state::CoverPool`] account.
pub const POOL_SEED: &[u8] = b"cover_pool";

/// PDA seed for the pool's token vault authority.
pub const VAULT_AUTHORITY_SEED: &[u8] = b"vault_authority";

/// PDA seed for an [`crate::state::LpPosition`] account.
pub const LP_POSITION_SEED: &[u8] = b"lp_position";

/// Initial shares minted per asset unit on the first deposit (virtual-share
/// offset that hardens the vault against the classic first-depositor inflation
/// attack, as popularised by ERC-4626 hardening guidance).
pub const INITIAL_SHARE_RATE: u64 = 1_000;

/// Minimum liquidity permanently locked on first deposit (burned shares) so the
/// share supply can never return to zero while assets remain.
pub const MINIMUM_LIQUIDITY: u64 = 1_000;
