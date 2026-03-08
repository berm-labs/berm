//! # berm-cover-engine
//!
//! The parametric trigger engine at the heart of the Berm cover protocol.
//!
//! Traditional discretionary cover pays only after a slow, adversarial claims
//! process. Parametric cover instead settles automatically the moment an
//! objective, oracle-observable condition is met -- the model studied in the
//! index-based parametric settlement literature and applied on-chain by this
//! crate. The engine
//! evaluates five cover types ([`cover_type::CoverType`]):
//!
//! | Cover type   | Parametric trigger                                            |
//! |--------------|---------------------------------------------------------------|
//! | Exploit      | TVL collapse beyond a threshold dominated by outflow          |
//! | Depeg        | Stablecoin price out of its peg band for N consecutive slots  |
//! | Slashing     | Covered validator stake slashed beyond a minimum fraction     |
//! | Liquidation  | Lending-market liquidation, absorbing a share of the penalty  |
//! | Oracle       | Cross-source price divergence beyond tolerance, persisted     |
//!
//! The flow is: [`trigger::MarketSnapshot`] (observations) ->
//! [`trigger::TriggerEngine`] (per-type evaluation) -> [`payout`] (solvency-aware
//! settlement) -> [`keeper`] (orchestration). The engine is pure and integer-only;
//! the `berm-keeper` binary wires it to live RPC data.
//!
//! Oracle aggregation and underwriting are delegated to the sibling crates
//! [`berm_oracle_adapter`] and [`berm_risk_underwriter`].

#![forbid(unsafe_code)]
#![deny(missing_docs)]

pub mod cover_type;
pub mod integrations;
pub mod keeper;
pub mod payout;
pub mod policy;
pub mod trigger;

pub use cover_type::{CoverParams, CoverType};
pub use integrations::{decode as decode_liquidation, to_event as liquidation_to_event, LendingProtocol};
pub use keeper::{Keeper, KeeperTick};
pub use payout::{build_settlement, Payout, SettlementBatch};
pub use policy::{Policy, PolicyStatus};
pub use trigger::{MarketSnapshot, TriggerEngine, TriggerOutcome};

// Re-export the dependency crates so downstream tooling (SDK, keeper binary) can
// reach the whole backend through one import root.
pub use berm_oracle_adapter as oracle;
pub use berm_risk_underwriter as underwriter;

/// Build the engine's default policy parameters for every cover type, used to
/// seed a fresh marketplace deployment.
pub fn default_params() -> Vec<CoverParams> {
    CoverType::all()
        .iter()
        .map(|t| CoverParams::default_for(*t))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_params_cover_all_types() {
        let params = default_params();
        assert_eq!(params.len(), 5);
        for (p, t) in params.iter().zip(CoverType::all()) {
            assert_eq!(p.cover_type(), t);
        }
    }

    #[test]
    fn decoded_liquidation_drives_liquidation_cover() {
        // Decode a real Solend LiquidateObligation instruction, value it via an
        // oracle price, and confirm it triggers a LiquidationCover policy.
        use integrations::{decode, solend, to_event, LendingProtocol};
        use trigger::MarketSnapshot;

        let ix = solend::encode_liquidate(1_000_000); // 1 token @ 6 decimals
        let raw = decode(LendingProtocol::Solend, &ix).expect("decodes");
        let event = to_event(&raw, 50_000, solend::DEFAULT_LIQUIDATION_BONUS_BPS, 42);
        // $500 notional, 5% bonus -> $25 penalty.
        assert_eq!(event.notional, 50_000);
        assert_eq!(event.penalty, 2_500);

        let mut snap = MarketSnapshot::at(42);
        snap.liquidation.insert("Solend:acct1".into(), event);
        let policy = Policy {
            id: 99,
            holder: "H".into(),
            subject: "Solend:acct1".into(),
            coverage: 1_000_000,
            params: CoverParams::default_for(CoverType::Liquidation),
            start_slot: 0,
            end_slot: 1_000,
            status: PolicyStatus::Active,
        };
        let outcome = TriggerEngine.evaluate(&policy, &snap).expect("triggers");
        assert_eq!(outcome.cover_type, CoverType::Liquidation);
        assert_eq!(outcome.payout, 2_000); // 80% of the $25 penalty
    }

    #[test]
    fn engine_dispatches_each_cover_type() {
        // A snapshot crafted to trigger each cover type confirms exhaustive
        // dispatch wiring across all five evaluators.
        use trigger::{
            DivergenceReading, LiquidationEvent, PegReading, SlashEvent, TvlReading,
        };
        let engine = TriggerEngine;
        let mut snap = MarketSnapshot::at(10);
        snap.tvl.insert(
            "P".into(),
            TvlReading {
                window_start: 100,
                current: 50,
                outflow: 50,
            },
        );
        snap.peg.insert(
            "USDC/USD".into(),
            PegReading {
                price: 90_000_000,
                expo: -8,
                out_of_band_slots: 5_000,
            },
        );
        snap.slashing.insert(
            "V".into(),
            SlashEvent {
                stake_before: 1_000,
                slashed: 100,
                epoch: 1,
            },
        );
        snap.liquidation.insert(
            "L".into(),
            LiquidationEvent {
                notional: 1_000_000,
                penalty: 100_000,
                slot: 9,
            },
        );
        snap.divergence.insert(
            "SOL/USD".into(),
            DivergenceReading {
                observed_bps: 800,
                persisted_slots: 5,
            },
        );

        for (t, subject) in [
            (CoverType::Exploit, "P"),
            (CoverType::Depeg, "USDC/USD"),
            (CoverType::Slashing, "V"),
            (CoverType::Liquidation, "L"),
            (CoverType::Oracle, "SOL/USD"),
        ] {
            let policy = Policy {
                id: 1,
                holder: "H".into(),
                subject: subject.into(),
                coverage: 1_000_000,
                params: CoverParams::default_for(t),
                start_slot: 0,
                end_slot: 1_000,
                status: PolicyStatus::Active,
            };
            let outcome = engine.evaluate(&policy, &snap);
            assert!(outcome.is_some(), "cover type {t:?} failed to trigger");
            assert_eq!(outcome.unwrap().cover_type, t);
        }
    }
}
