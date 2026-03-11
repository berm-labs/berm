//! LiquidationCover trigger.
//!
//! Borrowers on Marginfi / Kamino / Solend lose the liquidation penalty when
//! their position is liquidated. This cover absorbs a configurable fraction
//! (`absorb_bps`) of that realised penalty, softening cascade liquidations
//! during volatile markets. The trigger requires a minimum liquidated notional
//! so dust liquidations do not generate payouts.

use crate::cover_type::{LiquidationParams, BPS};
use crate::policy::Policy;

use super::{LiquidationEvent, MarketSnapshot, TriggerOutcome};

/// Portion of the penalty the cover absorbs (USD cents).
pub fn absorbed_loss(params: &LiquidationParams, event: &LiquidationEvent) -> u64 {
    ((event.penalty as u128 * params.absorb_bps as u128) / BPS as u128) as u64
}

/// Whether the event meets the liquidation trigger condition.
pub fn is_liquidation(params: &LiquidationParams, event: &LiquidationEvent) -> bool {
    event.notional >= params.min_notional && event.penalty > 0
}

/// Evaluate a liquidation policy against a snapshot.
pub fn evaluate(
    policy: &Policy,
    params: &LiquidationParams,
    snap: &MarketSnapshot,
) -> Option<TriggerOutcome> {
    let event = snap.liquidation.get(&policy.subject)?;
    if !is_liquidation(params, event) {
        return None;
    }
    let payout = absorbed_loss(params, event);
    if payout == 0 {
        return None;
    }
    Some(TriggerOutcome::new(
        policy,
        payout,
        snap.slot,
        format!(
            "liquidation of {} cents on {} (penalty {} cents, absorbed {} bps)",
            event.notional, policy.subject, event.penalty, params.absorb_bps
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
            id: 4,
            holder: "H".into(),
            subject: "Kamino:acct9".into(),
            coverage: 1_000_000,
            params: CoverParams::Liquidation(LiquidationParams::default()),
            start_slot: 0,
            end_slot: 100_000,
            status: PolicyStatus::Active,
        }
    }

    #[test]
    fn absorbs_configured_fraction_of_penalty() {
        let mut snap = MarketSnapshot::at(50);
        snap.liquidation.insert(
            "Kamino:acct9".into(),
            LiquidationEvent {
                notional: 500_000,
                penalty: 50_000,
                slot: 49,
            },
        );
        let o = evaluate(&policy(), &LiquidationParams::default(), &snap).unwrap();
        assert_eq!(o.payout, 40_000); // 80% of 50,000
    }

    #[test]
    fn ignores_dust_liquidation() {
        let mut snap = MarketSnapshot::at(50);
        snap.liquidation.insert(
            "Kamino:acct9".into(),
            LiquidationEvent {
                notional: 100, // below min_notional
                penalty: 10,
                slot: 49,
            },
        );
        assert!(evaluate(&policy(), &LiquidationParams::default(), &snap).is_none());
    }
}
