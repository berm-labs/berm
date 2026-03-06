//! The five parametric cover types.
//!
//! Each variant carries the parameters that define its trigger condition. A
//! parametric policy pays out when an *objective, oracle-observable* condition is
//! met -- there is no claims adjuster. The parameter sets here are what make each
//! cover type objectively testable by [`crate::trigger`].

use serde::{Deserialize, Serialize};

/// Basis-point denominator.
pub const BPS: u64 = 10_000;

/// The cover types offered by the protocol. Each maps to one breakwater block in
/// the UI and one trigger evaluator in the engine.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CoverType {
    /// Smart-contract exploit (TVL collapse + abnormal outflow).
    Exploit,
    /// Stablecoin depeg (price leaves the peg band for N slots).
    Depeg,
    /// LST validator slashing (stake slashed in an epoch).
    Slashing,
    /// Lending-market liquidation loss absorption.
    Liquidation,
    /// Oracle failure (cross-source divergence beyond tolerance).
    Oracle,
}

impl CoverType {
    /// Stable machine label.
    pub fn label(&self) -> &'static str {
        match self {
            CoverType::Exploit => "exploit",
            CoverType::Depeg => "depeg",
            CoverType::Slashing => "slashing",
            CoverType::Liquidation => "liquidation",
            CoverType::Oracle => "oracle",
        }
    }

    /// All five variants, in UI / pool order.
    pub fn all() -> [CoverType; 5] {
        [
            CoverType::Exploit,
            CoverType::Depeg,
            CoverType::Slashing,
            CoverType::Liquidation,
            CoverType::Oracle,
        ]
    }
}

/// Parameters for [`CoverType::Exploit`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExploitParams {
    /// TVL drop (bps) within the window that arms the cover.
    pub tvl_drop_bps: u64,
    /// Observation window in slots over which the drop is measured.
    pub window_slots: u64,
    /// Outflow concentration (bps of TVL leaving in the window) confirming an
    /// exploit rather than ordinary redemptions.
    pub outflow_bps: u64,
}

impl Default for ExploitParams {
    fn default() -> Self {
        Self {
            tvl_drop_bps: 3_000, // 30% TVL collapse
            window_slots: 9_000, // ~1 hour
            outflow_bps: 2_500,
        }
    }
}

/// Parameters for [`CoverType::Depeg`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct DepegParams {
    /// Lower peg bound mantissa at `expo` (e.g. 0.95).
    pub lower_bound: i64,
    /// Upper peg bound mantissa at `expo` (e.g. 1.05).
    pub upper_bound: i64,
    /// Exponent of the bounds.
    pub expo: i32,
    /// Consecutive slots the price must stay out of band to confirm a depeg.
    pub persistence_slots: u64,
}

impl Default for DepegParams {
    fn default() -> Self {
        Self {
            lower_bound: 95_000_000, // 0.95 at expo -8
            upper_bound: 105_000_000, // 1.05 at expo -8
            expo: -8,
            persistence_slots: 1_200, // ~8 min
        }
    }
}

/// Parameters for [`CoverType::Slashing`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SlashingParams {
    /// Minimum slashed fraction of staked lamports (bps) that arms the cover.
    pub min_slash_bps: u64,
}

impl Default for SlashingParams {
    fn default() -> Self {
        Self { min_slash_bps: 10 } // 0.1% slash
    }
}

/// Parameters for [`CoverType::Liquidation`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct LiquidationParams {
    /// Fraction of the realised liquidation penalty the cover absorbs (bps).
    pub absorb_bps: u64,
    /// Minimum liquidated notional (USD cents) below which no payout is made.
    pub min_notional: u64,
}

impl Default for LiquidationParams {
    fn default() -> Self {
        Self {
            absorb_bps: 8_000, // absorb 80% of the penalty
            min_notional: 10_000, // $100
        }
    }
}

/// Parameters for [`CoverType::Oracle`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct OracleParams {
    /// Divergence (bps) between sources that arms the cover.
    pub divergence_bps: u64,
}

impl Default for OracleParams {
    fn default() -> Self {
        Self {
            divergence_bps: 200, // 2% cross-source disagreement
        }
    }
}

/// Tagged union of per-type parameters carried by a [`crate::policy::Policy`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CoverParams {
    /// Exploit cover parameters.
    Exploit(ExploitParams),
    /// Depeg cover parameters.
    Depeg(DepegParams),
    /// Slashing cover parameters.
    Slashing(SlashingParams),
    /// Liquidation cover parameters.
    Liquidation(LiquidationParams),
    /// Oracle cover parameters.
    Oracle(OracleParams),
}

impl CoverParams {
    /// The cover type these parameters belong to.
    pub fn cover_type(&self) -> CoverType {
        match self {
            CoverParams::Exploit(_) => CoverType::Exploit,
            CoverParams::Depeg(_) => CoverType::Depeg,
            CoverParams::Slashing(_) => CoverType::Slashing,
            CoverParams::Liquidation(_) => CoverType::Liquidation,
            CoverParams::Oracle(_) => CoverType::Oracle,
        }
    }

    /// Default parameter set for a cover type.
    pub fn default_for(t: CoverType) -> CoverParams {
        match t {
            CoverType::Exploit => CoverParams::Exploit(ExploitParams::default()),
            CoverType::Depeg => CoverParams::Depeg(DepegParams::default()),
            CoverType::Slashing => CoverParams::Slashing(SlashingParams::default()),
            CoverType::Liquidation => CoverParams::Liquidation(LiquidationParams::default()),
            CoverType::Oracle => CoverParams::Oracle(OracleParams::default()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_returns_five_distinct_types() {
        let all = CoverType::all();
        assert_eq!(all.len(), 5);
        let labels: std::collections::HashSet<_> = all.iter().map(|t| t.label()).collect();
        assert_eq!(labels.len(), 5);
    }

    #[test]
    fn params_roundtrip_their_type() {
        for t in CoverType::all() {
            assert_eq!(CoverParams::default_for(t).cover_type(), t);
        }
    }

    #[test]
    fn depeg_band_is_ordered() {
        let p = DepegParams::default();
        assert!(p.lower_bound < p.upper_bound);
    }
}
