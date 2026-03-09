//! Cover policies -- the unit the engine evaluates.

use serde::{Deserialize, Serialize};

use crate::cover_type::{CoverParams, CoverType};

/// Lifecycle status of a policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PolicyStatus {
    /// Active and eligible to trigger.
    Active,
    /// A parametric condition fired; awaiting / settled payout.
    Triggered,
    /// Past expiry without triggering.
    Expired,
    /// Cancelled by the holder before expiry.
    Cancelled,
}

/// A single cover policy held by a protected wallet.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Policy {
    /// Unique policy identifier.
    pub id: u64,
    /// The wallet protected by this policy (base58 pubkey).
    pub holder: String,
    /// The subject being covered (protocol name, asset symbol, or validator).
    pub subject: String,
    /// Maximum payout this policy can produce (USD cents).
    pub coverage: u64,
    /// Trigger parameters (also fixes the cover type).
    pub params: CoverParams,
    /// Slot at which cover begins.
    pub start_slot: u64,
    /// Slot at which cover ends.
    pub end_slot: u64,
    /// Current status.
    pub status: PolicyStatus,
}

impl Policy {
    /// The cover type of this policy.
    pub fn cover_type(&self) -> CoverType {
        self.params.cover_type()
    }

    /// Whether the policy is in force at `slot`.
    pub fn is_active_at(&self, slot: u64) -> bool {
        self.status == PolicyStatus::Active && slot >= self.start_slot && slot <= self.end_slot
    }

    /// Whether the policy has lapsed (past end and still active).
    pub fn is_expired_at(&self, slot: u64) -> bool {
        slot > self.end_slot
    }

    /// Mark the policy triggered, returning the previous status.
    pub fn mark_triggered(&mut self) -> PolicyStatus {
        let prev = self.status;
        self.status = PolicyStatus::Triggered;
        prev
    }

    /// Remaining cover duration in slots at `slot` (0 if lapsed).
    pub fn remaining_slots(&self, slot: u64) -> u64 {
        self.end_slot.saturating_sub(slot)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cover_type::DepegParams;

    fn policy() -> Policy {
        Policy {
            id: 1,
            holder: "Hodler1111111111111111111111111111111111111".into(),
            subject: "USDC/USD".into(),
            coverage: 1_000_000,
            params: CoverParams::Depeg(DepegParams::default()),
            start_slot: 100,
            end_slot: 200,
            status: PolicyStatus::Active,
        }
    }

    #[test]
    fn active_window_is_inclusive() {
        let p = policy();
        assert!(!p.is_active_at(99));
        assert!(p.is_active_at(100));
        assert!(p.is_active_at(200));
        assert!(!p.is_active_at(201));
    }

    #[test]
    fn marking_triggered_changes_status() {
        let mut p = policy();
        assert_eq!(p.mark_triggered(), PolicyStatus::Active);
        assert_eq!(p.status, PolicyStatus::Triggered);
        assert!(!p.is_active_at(150));
    }

    #[test]
    fn cover_type_derives_from_params() {
        assert_eq!(policy().cover_type(), CoverType::Depeg);
    }
}
