//! Dispute-governance math (pure, unit-tested).
//!
//! Most claims settle automatically on an oracle trigger. A minority -- where
//! the parametric signal is ambiguous, or a payout is contested -- escalate to a
//! token-weighted vote, the model Nexus Mutual uses for its claims assessment.
//! This module holds the quorum and tally arithmetic so it can be verified in
//! isolation from the on-chain account plumbing.

/// Basis-point denominator.
pub const BPS: u64 = 10_000;

/// Outcome of a finalised dispute vote.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum VoteOutcome {
    /// The claim is approved for payout.
    Approve,
    /// The claim is rejected.
    Reject,
    /// Quorum was not reached; the dispute is void (claim rejected by default).
    NoQuorum,
}

/// Running vote tally (token-weighted).
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct Tally {
    /// Voting weight in favour of payout.
    pub approve_weight: u64,
    /// Voting weight against payout.
    pub reject_weight: u64,
}

impl Tally {
    /// Total weight cast.
    pub fn total(&self) -> u64 {
        self.approve_weight.saturating_add(self.reject_weight)
    }

    /// Add a ballot.
    pub fn add(&mut self, approve: bool, weight: u64) {
        if approve {
            self.approve_weight = self.approve_weight.saturating_add(weight);
        } else {
            self.reject_weight = self.reject_weight.saturating_add(weight);
        }
    }

    /// Approval share of cast weight in basis points.
    pub fn approval_bps(&self) -> u64 {
        let total = self.total();
        if total == 0 {
            return 0;
        }
        ((self.approve_weight as u128 * BPS as u128) / total as u128) as u64
    }
}

/// Whether quorum is met: cast weight must reach `quorum_bps` of `eligible`.
pub fn quorum_met(cast_weight: u64, eligible_weight: u64, quorum_bps: u64) -> bool {
    if eligible_weight == 0 {
        return false;
    }
    let cast_bps = ((cast_weight as u128 * BPS as u128) / eligible_weight as u128) as u64;
    cast_bps >= quorum_bps
}

/// Resolve a dispute given the tally, the eligible weight, and the thresholds.
pub fn resolve(
    tally: &Tally,
    eligible_weight: u64,
    quorum_bps: u64,
    approval_threshold_bps: u64,
) -> VoteOutcome {
    if !quorum_met(tally.total(), eligible_weight, quorum_bps) {
        return VoteOutcome::NoQuorum;
    }
    if tally.approval_bps() >= approval_threshold_bps {
        VoteOutcome::Approve
    } else {
        VoteOutcome::Reject
    }
}

/// Slash amount (asset units) for an underwriter whose assessment was wrong:
/// `slash_bps` of their stake, capped at the stake itself.
pub fn slash_amount(stake: u64, slash_bps: u64) -> u64 {
    (((stake as u128) * slash_bps.min(BPS) as u128) / BPS as u128) as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tally_tracks_approval_share() {
        let mut t = Tally::default();
        t.add(true, 700);
        t.add(false, 300);
        assert_eq!(t.total(), 1_000);
        assert_eq!(t.approval_bps(), 7_000);
    }

    #[test]
    fn quorum_requires_minimum_participation() {
        assert!(quorum_met(400, 1_000, 4_000));
        assert!(!quorum_met(399, 1_000, 4_000));
        assert!(!quorum_met(100, 0, 4_000));
    }

    #[test]
    fn resolve_paths() {
        let mut approve = Tally::default();
        approve.add(true, 600);
        approve.add(false, 100);
        assert_eq!(
            resolve(&approve, 1_000, 4_000, 6_000),
            VoteOutcome::Approve
        );

        let mut reject = Tally::default();
        reject.add(true, 100);
        reject.add(false, 600);
        assert_eq!(resolve(&reject, 1_000, 4_000, 6_000), VoteOutcome::Reject);

        let mut thin = Tally::default();
        thin.add(true, 100);
        assert_eq!(resolve(&thin, 1_000, 4_000, 6_000), VoteOutcome::NoQuorum);
    }

    #[test]
    fn slash_is_capped() {
        assert_eq!(slash_amount(1_000_000, 2_000), 200_000);
        assert_eq!(slash_amount(1_000_000, 20_000), 1_000_000);
    }
}
