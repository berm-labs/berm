//! SlashingCover trigger.
//!
//! Liquid-staking-token holders are exposed to validator slashing: when a
//! validator the LST delegates to is slashed in an epoch, the LST's backing
//! shrinks and holders absorb the loss pro-rata. The trigger fires when the
//! slashed fraction of a covered validator's stake exceeds `min_slash_bps`, and
//! the payout reimburses the holder's pro-rata share of the slashed value up to
//! the coverage limit.

use crate::cover_type::{SlashingParams, BPS};
use crate::policy::Policy;

use super::{MarketSnapshot, SlashEvent, TriggerOutcome};

/// Slashed fraction of stake in bps.
pub fn slash_bps(event: &SlashEvent) -> u64 {
    if event.stake_before == 0 {
        return 0;
    }
    ((event.slashed as u128 * BPS as u128) / event.stake_before as u128) as u64
}

/// Whether the event meets the slashing trigger condition.
pub fn is_slashing(params: &SlashingParams, event: &SlashEvent) -> bool {
    slash_bps(event) >= params.min_slash_bps
}

/// Evaluate a slashing policy against a snapshot.
pub fn evaluate(
    policy: &Policy,
    params: &SlashingParams,
    snap: &MarketSnapshot,
) -> Option<TriggerOutcome> {
    let event = snap.slashing.get(&policy.subject)?;
    if !is_slashing(params, event) {
        return None;
    }
    let bps = slash_bps(event);
    // Holder's loss is their coverage scaled by the slashed fraction.
    let payout = ((policy.coverage as u128 * bps as u128) / BPS as u128) as u64;
    Some(TriggerOutcome::new(
        policy,
        payout,
        snap.slot,
        format!(
            "{} slashed {bps} bps in epoch {}",
            policy.subject, event.epoch
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
            id: 3,
            holder: "H".into(),
            subject: "mSOL-validator-X".into(),
            coverage: 2_000_000,
            params: CoverParams::Slashing(SlashingParams::default()),
            start_slot: 0,
            end_slot: 100_000,
            status: PolicyStatus::Active,
        }
    }

    #[test]
    fn fires_on_slash_above_threshold() {
        let mut snap = MarketSnapshot::at(50);
        snap.slashing.insert(
            "mSOL-validator-X".into(),
            SlashEvent {
                stake_before: 1_000_000_000,
                slashed: 5_000_000, // 0.5% = 50 bps
                epoch: 600,
            },
        );
        let o = evaluate(&policy(), &SlashingParams::default(), &snap).unwrap();
        assert_eq!(o.payout, 10_000); // 50 bps of 2,000,000
    }

    #[test]
    fn ignores_dust_slash() {
        let mut snap = MarketSnapshot::at(50);
        snap.slashing.insert(
            "mSOL-validator-X".into(),
            SlashEvent {
                stake_before: 1_000_000_000,
                slashed: 100, // far below 10 bps
                epoch: 600,
            },
        );
        assert!(evaluate(&policy(), &SlashingParams::default(), &snap).is_none());
    }
}
