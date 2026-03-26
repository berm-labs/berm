//! On-chain cover-type enumeration.
//!
//! The ordinals here MUST match the off-chain `berm_cover_engine::CoverType`
//! discriminants so a policy issued on chain maps to the correct keeper
//! evaluator. The mapping is asserted in the engine's integration tests and
//! documented in `docs/cover-spec.md`.

use anchor_lang::prelude::*;

/// The five parametric cover types, in protocol order.
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub enum CoverType {
    /// Smart-contract exploit cover.
    Exploit,
    /// Stablecoin depeg cover.
    Depeg,
    /// Validator slashing cover.
    Slashing,
    /// Lending-market liquidation cover.
    Liquidation,
    /// Oracle-divergence cover.
    Oracle,
}

impl CoverType {
    /// Stable ordinal used as a PDA seed byte and engine discriminant.
    pub fn ordinal(&self) -> u8 {
        match self {
            CoverType::Exploit => 0,
            CoverType::Depeg => 1,
            CoverType::Slashing => 2,
            CoverType::Liquidation => 3,
            CoverType::Oracle => 4,
        }
    }

    /// Reconstruct from an ordinal byte.
    pub fn from_ordinal(b: u8) -> Option<CoverType> {
        match b {
            0 => Some(CoverType::Exploit),
            1 => Some(CoverType::Depeg),
            2 => Some(CoverType::Slashing),
            3 => Some(CoverType::Liquidation),
            4 => Some(CoverType::Oracle),
            _ => None,
        }
    }

    /// Whether this cover type settles against a price oracle (Depeg, Oracle).
    /// Used to decide whether an oracle account must be supplied at purchase.
    pub fn is_price_based(&self) -> bool {
        matches!(self, CoverType::Depeg | CoverType::Oracle)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ordinals_round_trip() {
        for b in 0u8..5 {
            assert_eq!(CoverType::from_ordinal(b).unwrap().ordinal(), b);
        }
        assert!(CoverType::from_ordinal(5).is_none());
    }

    #[test]
    fn price_based_flags() {
        assert!(CoverType::Depeg.is_price_based());
        assert!(CoverType::Oracle.is_price_based());
        assert!(!CoverType::Slashing.is_price_based());
    }
}
