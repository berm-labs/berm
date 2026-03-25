//! Individual risk factors.
//!
//! Each factor maps an observable protocol signal onto a 0..=1000 sub-score
//! where **higher means safer**. The composite score in [`crate::score`] is a
//! weighted blend of these. Splitting the factors out keeps the model auditable:
//! every input's contribution can be inspected in isolation, the way Nexus
//! Mutual's risk assessors decompose a listing.

use crate::protocol::ProtocolProfile;

/// Maximum value any factor (and the composite) can take.
pub const FACTOR_MAX: u32 = 1000;

/// TVL factor: larger, battle-tested capital pools are *safer* because they have
/// survived more adversarial pressure (Lindy effect). Scored on a log-style
/// ladder rather than linearly so a 10x TVL difference is not a 10x score.
pub fn tvl_factor(p: &ProtocolProfile) -> u32 {
    let tvl = p.tvl_usd();
    match tvl {
        t if t >= 1_000_000_000 => 1000,
        t if t >= 500_000_000 => 900,
        t if t >= 100_000_000 => 780,
        t if t >= 50_000_000 => 650,
        t if t >= 10_000_000 => 500,
        t if t >= 1_000_000 => 320,
        _ => 150,
    }
}

/// Audit factor: rewards multiple clean audits of the *current* code, penalises
/// open high findings.
pub fn audit_factor(p: &ProtocolProfile) -> u32 {
    let clean = p.clean_audit_count() as u32;
    let base = match clean {
        0 => 200,
        1 => 600,
        2 => 820,
        _ => 950,
    };
    let penalty = (p.open_high_findings() * 120).min(base);
    base - penalty
}

/// Code-complexity factor: more SLOC and higher cyclomatic complexity widen the
/// attack surface, so the factor *decreases* with complexity.
pub fn complexity_factor(p: &ProtocolProfile) -> u32 {
    // Normalise SLOC: 5k SLOC -> ~safe, 100k SLOC -> penalised.
    let sloc_pen = (p.sloc / 200).min(500) as u32;
    let cx_pen = (p.complexity_score).min(400);
    FACTOR_MAX.saturating_sub(sloc_pen + cx_pen)
}

/// Reputation factor: operating longevity without incident, less recent
/// incidents. Encodes the empirical observation that incident probability
/// front-loads in a protocol's early life.
pub fn reputation_factor(p: &ProtocolProfile) -> u32 {
    let longevity = match p.days_live {
        d if d >= 730 => 1000,
        d if d >= 365 => 850,
        d if d >= 180 => 680,
        d if d >= 90 => 500,
        _ => 300,
    };
    let incident_pen = (p.past_incidents * 250).min(longevity);
    longevity - incident_pen
}

/// Clamp an arbitrary integer into the valid factor range.
pub fn clamp_factor(v: i64) -> u32 {
    v.clamp(0, FACTOR_MAX as i64) as u32
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::{AuditRecord, ProtocolCategory};

    fn base() -> ProtocolProfile {
        ProtocolProfile {
            name: "X".into(),
            category: ProtocolCategory::Lending,
            tvl_usd_cents: 200_000_000 * 100,
            audits: vec![AuditRecord::clean("A"), AuditRecord::clean("B")],
            sloc: 20_000,
            complexity_score: 150,
            days_live: 800,
            past_incidents: 0,
        }
    }

    #[test]
    fn tvl_factor_is_monotonic() {
        let mut small = base();
        small.tvl_usd_cents = 2_000_000 * 100;
        assert!(tvl_factor(&base()) > tvl_factor(&small));
    }

    #[test]
    fn open_findings_reduce_audit_factor() {
        let mut p = base();
        let clean = audit_factor(&p);
        p.audits[0].unresolved_high = 1;
        assert!(audit_factor(&p) < clean);
    }

    #[test]
    fn incidents_reduce_reputation() {
        let mut p = base();
        let clean = reputation_factor(&p);
        p.past_incidents = 2;
        assert!(reputation_factor(&p) < clean);
    }

    #[test]
    fn complexity_penalises_large_codebases() {
        let mut p = base();
        let small = complexity_factor(&p);
        p.sloc = 200_000;
        p.complexity_score = 400;
        assert!(complexity_factor(&p) < small);
    }
}
