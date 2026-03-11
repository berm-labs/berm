//! OracleCover trigger.
//!
//! When two independent oracles (Pyth and Switchboard, optionally Chainlink)
//! disagree about an asset's price beyond `divergence_bps`, lending markets that
//! read the higher feed can liquidate borrowers unfairly. OracleCover reimburses
//! that bad-liquidation loss. The trigger consumes the divergence reading the
//! keeper computes via [`berm_oracle_adapter::detect_divergence`]; payout scales
//! with how far the divergence exceeds the policy threshold.

use crate::cover_type::{OracleParams, BPS};
use crate::policy::Policy;

use super::{DivergenceReading, MarketSnapshot, TriggerOutcome};

/// Minimum slots a divergence must persist before it can arm the cover, guarding
/// against a single-slot crossing during a fast move.
pub const MIN_PERSISTENCE_SLOTS: u64 = 2;

/// Whether the reading meets the oracle-divergence trigger condition.
pub fn is_divergent(params: &OracleParams, reading: &DivergenceReading) -> bool {
    reading.observed_bps >= params.divergence_bps && reading.persisted_slots >= MIN_PERSISTENCE_SLOTS
}

/// Payout fraction: linear in the excess divergence over the threshold, capped
/// at full coverage when the divergence reaches 2x the threshold.
pub fn payout_for(policy: &Policy, params: &OracleParams, reading: &DivergenceReading) -> u64 {
    let excess = reading.observed_bps.saturating_sub(params.divergence_bps);
    let span = params.divergence_bps.max(1);
    let frac_bps = ((excess as u128 * BPS as u128) / span as u128).min(BPS as u128) as u64;
    ((policy.coverage as u128 * frac_bps as u128) / BPS as u128) as u64
}

/// Evaluate an oracle-cover policy against a snapshot.
pub fn evaluate(
    policy: &Policy,
    params: &OracleParams,
    snap: &MarketSnapshot,
) -> Option<TriggerOutcome> {
    let reading = snap.divergence.get(&policy.subject)?;
    if !is_divergent(params, reading) {
        return None;
    }
    let payout = payout_for(policy, params, reading).max(1);
    Some(TriggerOutcome::new(
        policy,
        payout,
        snap.slot,
        format!(
            "oracle divergence {} bps on {} persisted {} slots",
            reading.observed_bps, policy.subject, reading.persisted_slots
        ),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cover_type::CoverParams;
    use crate::policy::PolicyStatus;

    fn policy() -> Policy {
        Policy {
            id: 5,
            holder: "H".into(),
            subject: "SOL/USD".into(),
            coverage: 1_000_000,
            params: CoverParams::Oracle(OracleParams::default()),
            start_slot: 0,
            end_slot: 100_000,
            status: PolicyStatus::Active,
        }
    }

    #[test]
    fn fires_on_persistent_divergence() {
        let mut snap = MarketSnapshot::at(50);
        snap.divergence.insert(
            "SOL/USD".into(),
            DivergenceReading {
                observed_bps: 400, // 2x the 200 bps threshold -> full payout
                persisted_slots: 4,
            },
        );
        let o = evaluate(&policy(), &OracleParams::default(), &snap).unwrap();
        assert_eq!(o.payout, 1_000_000);
    }

    #[test]
    fn ignores_single_slot_crossing() {
        let mut snap = MarketSnapshot::at(50);
        snap.divergence.insert(
            "SOL/USD".into(),
            DivergenceReading {
                observed_bps: 400,
                persisted_slots: 1, // below MIN_PERSISTENCE_SLOTS
            },
        );
        assert!(evaluate(&policy(), &OracleParams::default(), &snap).is_none());
    }

    #[test]
    fn partial_payout_scales_with_excess() {
        let p = policy();
        let r = DivergenceReading {
            observed_bps: 300, // 50% over threshold
            persisted_slots: 4,
        };
        assert_eq!(payout_for(&p, &OracleParams::default(), &r), 500_000);
    }
}
