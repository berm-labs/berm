//! Protocol risk profiles -- the underwriting subject.
//!
//! Before the engine can price cover on a protocol it needs a structured view of
//! that protocol's risk surface. The fields here are the on-chain- and
//! off-chain-observable signals that Sherlock-style audit-grounded cover and the
//! Nexus Mutual risk-assessor model both rely on: capital at risk (TVL), audit
//! history, code surface, operating history, and incident record.

use serde::{Deserialize, Serialize};

/// Broad protocol category. Different categories carry structurally different
/// base hazards (a lending market can cascade-liquidate; a stablecoin can
/// depeg), which feeds the base-rate selection in [`crate::premium`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ProtocolCategory {
    /// Over-collateralised lending market (Marginfi / Kamino / Solend).
    Lending,
    /// Automated market maker / concentrated-liquidity DEX.
    Dex,
    /// Liquid staking token issuer.
    LiquidStaking,
    /// Stablecoin issuer.
    Stablecoin,
    /// Perpetuals / derivatives venue.
    Perps,
    /// Yield aggregator / vault.
    YieldVault,
}

impl ProtocolCategory {
    /// Annualised base hazard rate in basis points, before risk adjustment.
    /// Calibrated from published DeFi loss-event frequencies.
    pub fn base_rate_bps(&self) -> u64 {
        match self {
            ProtocolCategory::Lending => 450,
            ProtocolCategory::Dex => 350,
            ProtocolCategory::LiquidStaking => 250,
            ProtocolCategory::Stablecoin => 300,
            ProtocolCategory::Perps => 600,
            ProtocolCategory::YieldVault => 500,
        }
    }
}

/// A single completed security audit.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuditRecord {
    /// Auditing firm name.
    pub firm: String,
    /// Number of unresolved critical/high findings at publication.
    pub unresolved_high: u32,
    /// Whether the audit covered the currently deployed code revision.
    pub covers_current_code: bool,
}

impl AuditRecord {
    /// A clean audit covering current code with no open highs.
    pub fn clean(firm: impl Into<String>) -> Self {
        Self {
            firm: firm.into(),
            unresolved_high: 0,
            covers_current_code: true,
        }
    }
}

/// Structured risk profile of a protocol being underwritten.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProtocolProfile {
    /// Display name.
    pub name: String,
    /// Risk category.
    pub category: ProtocolCategory,
    /// Total value locked, in USD cents (integer, no floats in settlement math).
    pub tvl_usd_cents: u64,
    /// Completed audits.
    pub audits: Vec<AuditRecord>,
    /// Source lines of code (a proxy for attack surface / complexity).
    pub sloc: u64,
    /// Cyclomatic-complexity-weighted instruction count of the program.
    pub complexity_score: u32,
    /// Days the protocol has operated in production without a critical incident.
    pub days_live: u32,
    /// Count of past loss-causing incidents.
    pub past_incidents: u32,
}

impl ProtocolProfile {
    /// Number of audits that cover the current code and carry no open highs.
    pub fn clean_audit_count(&self) -> usize {
        self.audits
            .iter()
            .filter(|a| a.covers_current_code && a.unresolved_high == 0)
            .count()
    }

    /// Total unresolved high/critical findings across all audits.
    pub fn open_high_findings(&self) -> u32 {
        self.audits.iter().map(|a| a.unresolved_high).sum()
    }

    /// TVL in whole USD (cents truncated), for reporting.
    pub fn tvl_usd(&self) -> u64 {
        self.tvl_usd_cents / 100
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> ProtocolProfile {
        ProtocolProfile {
            name: "Kamino".into(),
            category: ProtocolCategory::Lending,
            tvl_usd_cents: 1_200_000_000 * 100,
            audits: vec![AuditRecord::clean("OtterSec"), AuditRecord::clean("Sec3")],
            sloc: 45_000,
            complexity_score: 320,
            days_live: 700,
            past_incidents: 0,
        }
    }

    #[test]
    fn counts_clean_audits() {
        assert_eq!(sample().clean_audit_count(), 2);
        assert_eq!(sample().open_high_findings(), 0);
    }

    #[test]
    fn lending_has_highest_lending_base_rate() {
        assert!(ProtocolCategory::Perps.base_rate_bps() > ProtocolCategory::Lending.base_rate_bps());
        assert!(
            ProtocolCategory::Lending.base_rate_bps()
                > ProtocolCategory::LiquidStaking.base_rate_bps()
        );
    }
}
