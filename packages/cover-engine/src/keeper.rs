//! Keeper orchestration.
//!
//! The keeper is the off-chain process that, every slot, pulls oracle data and
//! protocol telemetry into a [`MarketSnapshot`], evaluates every active policy
//! through the [`TriggerEngine`], and packages the triggered outcomes into a
//! [`SettlementBatch`] for on-chain submission to the `claim-resolver` program.
//!
//! This module holds the pure decision logic (no I/O) so it can be unit tested
//! deterministically; the [`crate::bin`] entrypoint wires it to RPC.

use serde::{Deserialize, Serialize};

use berm_oracle_adapter::{aggregate, FeedConfig, Observation};

use crate::payout::{build_settlement, SettlementBatch};
use crate::policy::Policy;
use crate::trigger::{MarketSnapshot, TriggerEngine, TriggerOutcome};

/// One pass of the keeper loop, with everything it produced for observability.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KeeperTick {
    /// Slot evaluated.
    pub slot: u64,
    /// Active policies considered.
    pub considered: usize,
    /// Outcomes that triggered this tick.
    pub outcomes: Vec<TriggerOutcome>,
    /// The settlement batch (after solvency haircut).
    pub settlement: SettlementBatch,
}

/// The keeper's in-memory state: the policy book and the pool's free capital.
#[derive(Debug, Clone, Default)]
pub struct Keeper {
    engine: TriggerEngine,
    /// Free capital (USD cents) currently available for payouts.
    pub available_capital: u64,
}

impl Keeper {
    /// Build a keeper with a known free-capital figure.
    pub fn new(available_capital: u64) -> Self {
        Self {
            engine: TriggerEngine,
            available_capital,
        }
    }

    /// Refresh the resolved oracle price for a feed, surfacing divergence so the
    /// caller can populate the snapshot's divergence map for `OracleCover`.
    pub fn resolve_feed(
        &self,
        cfg: &FeedConfig,
        observations: &[Observation],
        slot: u64,
    ) -> Result<i64, u64> {
        match aggregate(cfg, observations, slot) {
            Ok(agg) => Ok(agg.price.price),
            Err(berm_oracle_adapter::OracleError::Divergence { observed_bps, .. }) => {
                Err(observed_bps)
            }
            Err(_) => Err(0),
        }
    }

    /// Run one keeper tick over the policy book against a prepared snapshot.
    pub fn tick(&self, policies: &[Policy], snap: &MarketSnapshot) -> KeeperTick {
        let considered = policies.iter().filter(|p| p.is_active_at(snap.slot)).count();
        let outcomes = self.engine.evaluate_all(policies, snap);
        let with_recipients: Vec<(TriggerOutcome, String)> = outcomes
            .iter()
            .map(|o| {
                let recipient = policies
                    .iter()
                    .find(|p| p.id == o.policy_id)
                    .map(|p| p.holder.clone())
                    .unwrap_or_default();
                (o.clone(), recipient)
            })
            .collect();
        let settlement = build_settlement(&with_recipients, self.available_capital);
        KeeperTick {
            slot: snap.slot,
            considered,
            outcomes,
            settlement,
        }
    }

    /// Serialise a tick to JSON for the keeper's structured log / API.
    pub fn tick_to_json(tick: &KeeperTick) -> String {
        serde_json::to_string(tick).unwrap_or_else(|_| "{}".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cover_type::{CoverParams, DepegParams};
    use crate::policy::PolicyStatus;
    use crate::trigger::PegReading;

    fn depeg_policy(id: u64) -> Policy {
        Policy {
            id,
            holder: format!("Holder{id}"),
            subject: "USDC/USD".into(),
            coverage: 1_000_000,
            params: CoverParams::Depeg(DepegParams::default()),
            start_slot: 0,
            end_slot: 100_000,
            status: PolicyStatus::Active,
        }
    }

    #[test]
    fn tick_settles_triggered_policies() {
        let keeper = Keeper::new(5_000_000);
        let policies = vec![depeg_policy(1), depeg_policy(2)];
        let mut snap = MarketSnapshot::at(500);
        snap.peg.insert(
            "USDC/USD".into(),
            PegReading {
                price: 90_000_000,
                expo: -8,
                out_of_band_slots: 2_000,
            },
        );
        let tick = keeper.tick(&policies, &snap);
        assert_eq!(tick.considered, 2);
        assert_eq!(tick.outcomes.len(), 2);
        assert!(!tick.settlement.is_haircut());
        let json = Keeper::tick_to_json(&tick);
        assert!(json.contains("\"slot\":500"));
    }

    #[test]
    fn tick_applies_haircut_under_correlated_event() {
        let keeper = Keeper::new(200_000); // less than the two combined payouts
        let policies = vec![depeg_policy(1), depeg_policy(2)];
        let mut snap = MarketSnapshot::at(500);
        snap.peg.insert(
            "USDC/USD".into(),
            PegReading {
                price: 80_000_000, // deep depeg -> large payouts
                expo: -8,
                out_of_band_slots: 5_000,
            },
        );
        let tick = keeper.tick(&policies, &snap);
        assert!(tick.settlement.is_haircut());
        assert!(tick.settlement.total_net <= 500_000);
    }
}
