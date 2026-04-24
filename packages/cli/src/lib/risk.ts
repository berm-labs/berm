import type { WalletSnapshot } from "./solana.js";
import {
  COVER_TYPE_META,
  type CoverType,
} from "./constants.js";

export interface ExposureBreakdown {
  stableTokens: number;
  lstTokens: number;
  nativeSol: number;
  otherTokens: number;
}

export interface CoverRecommendation {
  coverType: CoverType;
  label: string;
  exposureUnits: number; // tokens or SOL depending on type
  exposureUnit: string;
  rationale: string;
  priority: "high" | "medium" | "low";
}

export interface RiskReport {
  owner: string;
  slot: number;
  // 0 (calm) -> 100 (storm)
  score: number;
  scores: {
    depeg: number;
    slashing: number;
    concentration: number;
  };
  exposure: ExposureBreakdown;
  concentrationIndex: number; // 0..1 Herfindahl over token classes
  recommendations: CoverRecommendation[];
  notes: string[];
}

function clamp(n: number, lo: number, hi: number): number {
  return Math.max(lo, Math.min(hi, n));
}

// Logistic-style saturation so that risk grows with exposure but never exceeds
// 100. `half` is the exposure level at which the sub-score reaches 50.
function saturate(value: number, half: number): number {
  if (value <= 0) return 0;
  return clamp((value / (value + half)) * 100, 0, 100);
}

// Herfindahl-Hirschman index over class weights, normalised to 0..1.
// 1.0 means everything sits in a single asset class (maximally concentrated).
function herfindahl(weights: number[]): number {
  const total = weights.reduce((a, b) => a + b, 0);
  if (total <= 0) return 0;
  return weights.reduce((acc, w) => acc + (w / total) ** 2, 0);
}

// Produces a parametric risk report purely from on-chain holdings. Stable and
// LST exposures drive depeg and slashing sub-scores; class concentration drives
// the concentration sub-score. The overall score weights the cover types by the
// inherent severity encoded in their base rates.
export function scoreWallet(snapshot: WalletSnapshot): RiskReport {
  const exposure: ExposureBreakdown = {
    stableTokens: 0,
    lstTokens: 0,
    nativeSol: snapshot.solBalance,
    otherTokens: 0,
  };

  for (const h of snapshot.holdings) {
    if (h.class === "stable") exposure.stableTokens += h.amount;
    else if (h.class === "lst") exposure.lstTokens += h.amount;
    else if (h.class === "native") exposure.nativeSol += h.amount;
    else exposure.otherTokens += h.amount;
  }

  // Sub-scores. `half` constants are tuned so a meaningful position registers.
  const depeg = saturate(exposure.stableTokens, 5_000);
  const slashing = saturate(exposure.lstTokens + exposure.nativeSol, 50);

  const classWeights = [
    exposure.stableTokens,
    exposure.lstTokens + exposure.nativeSol,
    exposure.otherTokens,
  ];
  const concentrationIndex = herfindahl(classWeights);
  const concentration = clamp(concentrationIndex * 100, 0, 100);

  // Weight cover-type severities by base rate to fold sub-scores into one.
  const wDepeg = COVER_TYPE_META.depeg.baseRateBps;
  const wSlash = COVER_TYPE_META.slashing.baseRateBps;
  const wConc = COVER_TYPE_META.liquidation.baseRateBps;
  const wTotal = wDepeg + wSlash + wConc;
  const score = Math.round(
    (depeg * wDepeg + slashing * wSlash + concentration * wConc) / wTotal,
  );

  const recommendations: CoverRecommendation[] = [];
  if (exposure.stableTokens > 0) {
    recommendations.push({
      coverType: "depeg",
      label: COVER_TYPE_META.depeg.label,
      exposureUnits: round(exposure.stableTokens),
      exposureUnit: "stable units",
      rationale: "Stablecoin balance is exposed to depeg below 0.95.",
      priority: exposure.stableTokens > 5_000 ? "high" : "medium",
    });
  }
  if (exposure.lstTokens > 0) {
    recommendations.push({
      coverType: "slashing",
      label: COVER_TYPE_META.slashing.label,
      exposureUnits: round(exposure.lstTokens),
      exposureUnit: "LST",
      rationale: "Liquid staking tokens are exposed to validator slashing.",
      priority: exposure.lstTokens > 100 ? "high" : "medium",
    });
  }
  if (exposure.lstTokens > 0 || exposure.otherTokens > 0) {
    recommendations.push({
      coverType: "oracle",
      label: COVER_TYPE_META.oracle.label,
      exposureUnits: round(exposure.lstTokens + exposure.otherTokens),
      exposureUnit: "priced assets",
      rationale: "Priced assets are exposed to oracle divergence mispricing.",
      priority: "low",
    });
  }

  const notes: string[] = [];
  notes.push(
    "Liquidation exposure depends on open lending positions. Connect a backend with `--api` to include Marginfi / Kamino / Solend positions.",
  );
  if (snapshot.holdings.length === 0 && snapshot.solBalance === 0) {
    notes.push("Wallet holds no SOL or token balances at the scanned slot.");
  }

  return {
    owner: snapshot.owner,
    slot: snapshot.slot,
    score,
    scores: { depeg, slashing, concentration },
    exposure,
    concentrationIndex,
    recommendations,
    notes,
  };
}

// Local premium estimator. Mirrors the on-chain pricing curve: annualised base
// rate scaled by duration and a risk multiplier derived from the wallet score.
// The authoritative quote comes from `/cover/quote`; this is the offline fallback.
export function estimatePremium(input: {
  coverType: CoverType;
  amountUsd: number;
  durationDays: number;
  riskScore?: number;
}): { premiumUsd: number; rateBps: number } {
  const meta = COVER_TYPE_META[input.coverType];
  const riskMultiplier = 1 + (clamp(input.riskScore ?? 0, 0, 100) / 100) * 0.75;
  const annualRate = (meta.baseRateBps / 10_000) * riskMultiplier;
  const durationFraction = input.durationDays / 365;
  const premiumUsd = input.amountUsd * annualRate * durationFraction;
  const rateBps = Math.round((premiumUsd / input.amountUsd) * 10_000);
  return { premiumUsd: round(premiumUsd), rateBps };
}

function round(n: number): number {
  return Math.round(n * 100) / 100;
}
