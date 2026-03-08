//! Payout settlement records.
//!
//! When a policy triggers, the engine produces a [`super::trigger::TriggerOutcome`].
//! Before the keeper submits the settlement transaction it batches outcomes into
//! a [`SettlementBatch`] and checks the aggregate against the pool's free
//! capital, so a correlated event (e.g. a market-wide depeg) cannot instruct the
//! pool to pay out more than it holds. If the batch exceeds available capital it
//! is settled pro-rata, the standard mutual-insolvency haircut.

use serde::{Deserialize, Serialize};

use crate::cover_type::BPS;
use crate::trigger::TriggerOutcome;

/// A finalised payout instruction for one policy.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Payout {
    /// Policy being paid.
    pub policy_id: u64,
    /// Holder receiving the payout (base58 pubkey).
    pub recipient: String,
    /// Gross amount the trigger computed (USD cents).
    pub gross: u64,
    /// Net amount after any pro-rata haircut (USD cents).
    pub net: u64,
}

/// A batch of payouts settled together against pool capital.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SettlementBatch {
    /// The individual payouts.
    pub payouts: Vec<Payout>,
    /// Total gross before haircut.
    pub total_gross: u64,
    /// Total net after haircut.
    pub total_net: u64,
    /// Pro-rata factor applied (bps; 10_000 = no haircut).
    pub haircut_factor_bps: u64,
}

/// Build a settlement batch from triggered outcomes, applying a pro-rata haircut
/// if the gross total exceeds `available_capital`.
pub fn build_settlement(
    outcomes: &[(TriggerOutcome, String)],
    available_capital: u64,
) -> SettlementBatch {
    let total_gross: u64 = outcomes.iter().map(|(o, _)| o.payout).sum();
    let haircut_factor_bps = if total_gross == 0 || total_gross <= available_capital {
        BPS
    } else {
        ((available_capital as u128 * BPS as u128) / total_gross as u128) as u64
    };
    let mut payouts = Vec::with_capacity(outcomes.len());
    let mut total_net = 0u64;
    for (o, recipient) in outcomes {
        let net = ((o.payout as u128 * haircut_factor_bps as u128) / BPS as u128) as u64;
        total_net += net;
        payouts.push(Payout {
            policy_id: o.policy_id,
            recipient: recipient.clone(),
            gross: o.payout,
            net,
        });
    }
    SettlementBatch {
        payouts,
        total_gross,
        total_net,
        haircut_factor_bps,
    }
}

impl SettlementBatch {
    /// Whether a solvency haircut was applied.
    pub fn is_haircut(&self) -> bool {
        self.haircut_factor_bps < BPS
    }

    /// Number of policies in the batch.
    pub fn len(&self) -> usize {
        self.payouts.len()
    }

    /// Whether the batch is empty.
    pub fn is_empty(&self) -> bool {
        self.payouts.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cover_type::CoverType;

    fn outcome(id: u64, payout: u64) -> (TriggerOutcome, String) {
        (
            TriggerOutcome {
                policy_id: id,
                cover_type: CoverType::Depeg,
                payout,
                slot: 1,
                reason: "x".into(),
            },
            format!("recipient{id}"),
        )
    }

    #[test]
    fn no_haircut_when_solvent() {
        let b = build_settlement(&[outcome(1, 100), outcome(2, 200)], 1_000);
        assert!(!b.is_haircut());
        assert_eq!(b.total_net, 300);
    }

    #[test]
    fn pro_rata_haircut_when_insolvent() {
        let b = build_settlement(&[outcome(1, 600), outcome(2, 600)], 600);
        assert!(b.is_haircut());
        assert_eq!(b.haircut_factor_bps, 5_000); // 50%
        assert_eq!(b.payouts[0].net, 300);
        assert_eq!(b.payouts[1].net, 300);
        assert!(b.total_net <= 600);
    }

    #[test]
    fn empty_batch_is_safe() {
        let b = build_settlement(&[], 1_000);
        assert!(b.is_empty());
        assert!(!b.is_haircut());
    }
}
