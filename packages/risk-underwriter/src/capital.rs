//! Capital adequacy.
//!
//! A cover pool is only as credible as its ability to pay. This module tracks
//! the pool's solvency the way a mutual tracks its minimum capital requirement
//! (MCR): the ratio of free capital to outstanding cover liability must stay
//! above a floor before new cover can be written. This is the on-chain solvency
//! invariant the `pool-vault` program enforces, modelled here for the off-chain
//! underwriter so quotes are pre-validated before a transaction is built.

use serde::{Deserialize, Serialize};

use crate::error::{UnderwriteError, UnderwriteResult};
use crate::model::BPS;

/// A snapshot of a cover pool's capital position.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct PoolCapital {
    /// Total capital deposited by LPs (USD cents).
    pub total_capital: u64,
    /// Sum of active coverage limits the pool is liable for (USD cents).
    pub outstanding_cover: u64,
    /// Premiums collected and not yet distributed (USD cents).
    pub unearned_premium: u64,
}

impl PoolCapital {
    /// Capital free to back new cover.
    pub fn free_capital(&self) -> u64 {
        self.total_capital.saturating_sub(self.locked_capital())
    }

    /// Capital locked behind outstanding cover at the configured backing ratio.
    /// The protocol backs each unit of cover with `BACKING_RATIO_BPS` of capital
    /// rather than 1:1, reflecting the low joint-probability of all policies
    /// triggering at once (the mutualisation premise).
    pub fn locked_capital(&self) -> u64 {
        ((self.outstanding_cover as u128 * BACKING_RATIO_BPS as u128) / BPS as u128) as u64
    }

    /// Capital adequacy ratio in basis points: free+locked vs outstanding cover.
    pub fn adequacy_bps(&self) -> u64 {
        if self.outstanding_cover == 0 {
            return u64::MAX;
        }
        ((self.total_capital as u128 * BPS as u128) / self.outstanding_cover as u128) as u64
    }

    /// Utilisation in basis points (locked / total).
    pub fn utilization_bps(&self) -> u64 {
        if self.total_capital == 0 {
            return BPS;
        }
        ((self.locked_capital() as u128 * BPS as u128) / self.total_capital as u128).min(BPS as u128)
            as u64
    }

    /// Verify the pool can absorb `new_cover` and stay above the MCR floor.
    pub fn can_underwrite(&self, new_cover: u64) -> UnderwriteResult<()> {
        let projected = PoolCapital {
            total_capital: self.total_capital,
            outstanding_cover: self.outstanding_cover.saturating_add(new_cover),
            unearned_premium: self.unearned_premium,
        };
        let ratio = projected.adequacy_bps();
        if ratio < MIN_ADEQUACY_BPS {
            return Err(UnderwriteError::Undercapitalised {
                ratio_bps: ratio,
                min_bps: MIN_ADEQUACY_BPS,
            });
        }
        let available = self.free_capital();
        let needed = ((new_cover as u128 * BACKING_RATIO_BPS as u128) / BPS as u128) as u64;
        if needed > available {
            return Err(UnderwriteError::InsufficientCapacity {
                requested: needed,
                available,
            });
        }
        Ok(())
    }
}

/// Fraction of each cover unit that must be backed by pool capital (bps).
pub const BACKING_RATIO_BPS: u64 = 4_000; // 40%

/// Minimum capital-adequacy ratio before new cover can be written (bps).
/// 12_000 bps = the pool must hold >= 1.2x its total outstanding cover limit.
pub const MIN_ADEQUACY_BPS: u64 = 12_000;

#[cfg(test)]
mod tests {
    use super::*;

    fn pool() -> PoolCapital {
        PoolCapital {
            total_capital: 10_000_000,
            outstanding_cover: 5_000_000,
            unearned_premium: 100_000,
        }
    }

    #[test]
    fn computes_locked_and_free() {
        let p = pool();
        assert_eq!(p.locked_capital(), 2_000_000); // 40% of 5M
        assert_eq!(p.free_capital(), 8_000_000);
    }

    #[test]
    fn adequacy_above_one_for_healthy_pool() {
        assert!(pool().adequacy_bps() >= MIN_ADEQUACY_BPS);
    }

    #[test]
    fn blocks_underwriting_when_it_breaches_mcr() {
        let p = pool();
        // Adding huge cover pushes adequacy below the floor.
        assert!(p.can_underwrite(20_000_000).is_err());
        // A modest policy is fine.
        assert!(p.can_underwrite(1_000_000).is_ok());
    }
}
