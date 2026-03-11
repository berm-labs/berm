//! The parametric trigger engine.
//!
//! The engine evaluates a [`crate::policy::Policy`] against a [`MarketSnapshot`]
//! -- the bundle of oracle prices, TVL readings, slashing events, and
//! liquidation events observed at a given slot -- and decides whether the
//! policy's objective trigger condition has been met. If it has, it returns a
//! [`TriggerOutcome`] carrying the computed payout.
//!
//! Each cover type has its own evaluator submodule. Adding a cover type means
//! adding a submodule and a match arm here -- the dispatch is exhaustive over
//! [`crate::cover_type::CoverType`], so the compiler enforces completeness.

pub mod depeg;
pub mod exploit;
pub mod liquidation;
pub mod oracle;
pub mod slashing;

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::cover_type::{CoverParams, CoverType};
use crate::policy::Policy;

/// A protocol TVL reading pair used by the exploit evaluator.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct TvlReading {
    /// TVL (USD cents) at the start of the observation window.
    pub window_start: u64,
    /// TVL (USD cents) now.
    pub current: u64,
    /// Net outflow (USD cents) observed across the window.
    pub outflow: u64,
}

/// A validator slashing observation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SlashEvent {
    /// Staked lamports before the slash.
    pub stake_before: u64,
    /// Lamports slashed in the epoch.
    pub slashed: u64,
    /// Epoch the slash occurred in.
    pub epoch: u64,
}

/// A liquidation observation from a lending market.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct LiquidationEvent {
    /// Notional liquidated (USD cents).
    pub notional: u64,
    /// Penalty charged to the borrower (USD cents).
    pub penalty: u64,
    /// Slot of the liquidation.
    pub slot: u64,
}

/// A divergence observation for an oracle feed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct DivergenceReading {
    /// Maximum cross-source divergence observed (bps).
    pub observed_bps: u64,
    /// Number of consecutive slots the divergence has persisted.
    pub persisted_slots: u64,
}

/// A depeg observation for a price feed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct PegReading {
    /// Aggregated price mantissa.
    pub price: i64,
    /// Exponent of `price`.
    pub expo: i32,
    /// Consecutive slots the price has been out of band.
    pub out_of_band_slots: u64,
}

/// Everything the engine observes at one slot, keyed by policy subject.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MarketSnapshot {
    /// Slot this snapshot was taken at.
    pub slot: u64,
    /// Protocol TVL readings keyed by protocol name.
    pub tvl: HashMap<String, TvlReading>,
    /// Peg readings keyed by asset symbol.
    pub peg: HashMap<String, PegReading>,
    /// Slash events keyed by validator / LST identifier.
    pub slashing: HashMap<String, SlashEvent>,
    /// Liquidation events keyed by market+account identifier.
    pub liquidation: HashMap<String, LiquidationEvent>,
    /// Divergence readings keyed by feed symbol.
    pub divergence: HashMap<String, DivergenceReading>,
}

impl MarketSnapshot {
    /// Construct an empty snapshot at a slot.
    pub fn at(slot: u64) -> Self {
        Self {
            slot,
            ..Default::default()
        }
    }
}

/// The result of a successful trigger evaluation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TriggerOutcome {
    /// Policy that triggered.
    pub policy_id: u64,
    /// Cover type that fired.
    pub cover_type: CoverType,
    /// Payout owed (USD cents), already capped at the policy coverage.
    pub payout: u64,
    /// Slot at which the trigger condition was confirmed.
    pub slot: u64,
    /// Human-readable explanation of why the trigger fired.
    pub reason: String,
}

impl TriggerOutcome {
    /// Build an outcome, clamping the payout to the policy coverage.
    pub fn new(policy: &Policy, payout: u64, slot: u64, reason: impl Into<String>) -> Self {
        Self {
            policy_id: policy.id,
            cover_type: policy.cover_type(),
            payout: payout.min(policy.coverage),
            slot,
            reason: reason.into(),
        }
    }
}

/// Stateless evaluator that dispatches to the per-type logic.
#[derive(Debug, Clone, Copy, Default)]
pub struct TriggerEngine;

impl TriggerEngine {
    /// Evaluate one policy against a snapshot. Returns `Some` if the policy's
    /// parametric condition fired, `None` otherwise. Inactive or expired
    /// policies never trigger.
    pub fn evaluate(&self, policy: &Policy, snap: &MarketSnapshot) -> Option<TriggerOutcome> {
        if !policy.is_active_at(snap.slot) {
            return None;
        }
        match &policy.params {
            CoverParams::Exploit(p) => exploit::evaluate(policy, p, snap),
            CoverParams::Depeg(p) => depeg::evaluate(policy, p, snap),
            CoverParams::Slashing(p) => slashing::evaluate(policy, p, snap),
            CoverParams::Liquidation(p) => liquidation::evaluate(policy, p, snap),
            CoverParams::Oracle(p) => oracle::evaluate(policy, p, snap),
        }
    }

    /// Evaluate a batch of policies, returning every outcome that fired.
    pub fn evaluate_all(&self, policies: &[Policy], snap: &MarketSnapshot) -> Vec<TriggerOutcome> {
        policies
            .iter()
            .filter_map(|p| self.evaluate(p, snap))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cover_type::{CoverParams, DepegParams};
    use crate::policy::PolicyStatus;

    fn policy() -> Policy {
        Policy {
            id: 7,
            holder: "H".into(),
            subject: "USDC/USD".into(),
            coverage: 500_000,
            params: CoverParams::Depeg(DepegParams::default()),
            start_slot: 0,
            end_slot: 1_000,
            status: PolicyStatus::Active,
        }
    }

    #[test]
    fn inactive_policy_never_triggers() {
        let eng = TriggerEngine;
        let mut p = policy();
        p.status = PolicyStatus::Cancelled;
        let snap = MarketSnapshot::at(10);
        assert!(eng.evaluate(&p, &snap).is_none());
    }

    #[test]
    fn outcome_caps_payout_at_coverage() {
        let p = policy();
        let o = TriggerOutcome::new(&p, 10_000_000, 5, "x");
        assert_eq!(o.payout, p.coverage);
    }
}
