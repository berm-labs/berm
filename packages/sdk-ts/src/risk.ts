import BN from "bn.js";
import { CoverType } from "./constants";
import { RiskScore, CoverQuote } from "./types";
import { clamp } from "./severity";

/** Raw protocol signals consumed by the risk model. */
export interface ProtocolSignals {
  protocol: string;
  /** Total value locked, in USD. */
  tvlUsd: number;
  /** Number of completed third-party audits. */
  auditCount: number;
  /** Code complexity index, 0 (simple) to 100 (complex). */
  codeComplexity: number;
  /** Reputation/activity index, 0 (unproven) to 100 (established). */
  reputation: number;
}

/**
 * Scores protocol risk from TVL, audit coverage, code complexity, and
 * reputation, then derives a recommended premium rate. Mirrors the
 * risk-underwriter crate so clients can quote without a round trip.
 */
export class RiskScorer {
  /** Component weights; sum to 1. */
  private static readonly WEIGHTS = {
    tvl: 0.25,
    audits: 0.3,
    codeComplexity: 0.25,
    reputation: 0.2,
  };

  /** Minimum and maximum annualized premium rate, basis points. */
  private static readonly MIN_RATE_BPS = 150;
  private static readonly MAX_RATE_BPS = 3500;

  /** Compute a composite 0-100 risk score and recommended premium rate. */
  score(signals: ProtocolSignals): RiskScore {
    // Larger TVL lowers relative risk (log-scaled, $1B as the safe anchor).
    const tvlRisk = clamp(
      100 - (Math.log10(Math.max(signals.tvlUsd, 1)) / 9) * 100,
      0,
      100
    );
    // More audits lower risk; 4+ audits approaches the floor.
    const auditRisk = clamp(100 - signals.auditCount * 25, 0, 100);
    const complexityRisk = clamp(signals.codeComplexity, 0, 100);
    const reputationRisk = clamp(100 - signals.reputation, 0, 100);

    const w = RiskScorer.WEIGHTS;
    const composite =
      tvlRisk * w.tvl +
      auditRisk * w.audits +
      complexityRisk * w.codeComplexity +
      reputationRisk * w.reputation;

    const score = Math.round(clamp(composite, 0, 100));
    const premiumRateBps = Math.round(
      RiskScorer.MIN_RATE_BPS +
        (score / 100) * (RiskScorer.MAX_RATE_BPS - RiskScorer.MIN_RATE_BPS)
    );

    return {
      protocol: signals.protocol,
      score,
      components: {
        tvl: Math.round(tvlRisk),
        audits: Math.round(auditRisk),
        codeComplexity: Math.round(complexityRisk),
        reputation: Math.round(reputationRisk),
      },
      premiumRateBps,
    };
  }

  /**
   * Quote the premium for a cover purchase given a risk score.
   * premium = coverAmount * rate * (durationDays / 365)
   */
  quote(
    coverType: CoverType,
    coverAmount: BN,
    durationDays: number,
    risk: RiskScore
  ): CoverQuote {
    // Pro-rate the annualized rate to the requested duration in basis points.
    const durationBps = Math.round((durationDays / 365) * 10000);
    const premium = coverAmount
      .mul(new BN(risk.premiumRateBps))
      .mul(new BN(durationBps))
      .div(new BN(10000))
      .div(new BN(10000));
    return {
      coverType,
      coverAmount,
      durationDays,
      premium,
      premiumRateBps: risk.premiumRateBps,
    };
  }
}
