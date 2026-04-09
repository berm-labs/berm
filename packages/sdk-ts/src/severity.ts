import BN from "bn.js";
import { CoverType, DEFAULT_COVER_PARAMS } from "./constants";

/** Clamp a number to the inclusive [lo, hi] range. */
export function clamp(value: number, lo: number, hi: number): number {
  return Math.max(lo, Math.min(hi, value));
}

/**
 * ExploitCover severity: normalized TVL drop above the trigger threshold.
 * severity = clamp((drop - T) / (1 - T), 0, 1)
 */
export function exploitSeverity(dropRatio: number, threshold?: number): number {
  const t = threshold ?? DEFAULT_COVER_PARAMS[CoverType.Exploit].threshold;
  if (dropRatio <= t) return 0;
  return clamp((dropRatio - t) / (1 - t), 0, 1);
}

/**
 * DepegCover severity: depeg depth past the [0.95, 1.05] band, normalized
 * to the saturation cap. severity = clamp(depth / Dmax, 0, 1)
 */
export function depegSeverity(price: number, severityCap?: number): number {
  const dMax = severityCap ?? DEFAULT_COVER_PARAMS[CoverType.Depeg].severityCap;
  const depth = Math.max(0.95 - price, price - 1.05, 0);
  return clamp(depth / dMax, 0, 1);
}

/** SlashingCover severity: the slashed proportion of staked principal. */
export function slashingSeverity(
  slashedStake: number,
  totalStaked: number
): number {
  if (totalStaked <= 0) return 0;
  return clamp(slashedStake / totalStaked, 0, 1);
}

/** LiquidationCover severity: realized loss over collateral at risk. */
export function liquidationSeverity(
  realizedLoss: number,
  collateralAtRisk: number
): number {
  if (collateralAtRisk <= 0) return 0;
  return clamp(realizedLoss / collateralAtRisk, 0, 1);
}

/**
 * OracleCover severity: divergence above the threshold, normalized.
 * severity = clamp((divergence - T) / T, 0, 1)
 */
export function oracleSeverity(divergence: number, threshold?: number): number {
  const t = threshold ?? DEFAULT_COVER_PARAMS[CoverType.Oracle].threshold;
  if (divergence <= t) return 0;
  return clamp((divergence - t) / t, 0, 1);
}

/**
 * Apply a severity ratio and cover ratio to a notional, bounded by the
 * cover amount. payout = min(C, r * severity * notional).
 *
 * @param coverAmount maximum payout C, in base units
 * @param coverRatioBps cover ratio r, in basis points (10000 = 100%)
 * @param severity severity ratio in [0, 1]
 * @param notional exposed notional, in base units
 */
export function payout(
  coverAmount: BN,
  coverRatioBps: number,
  severity: number,
  notional: BN
): BN {
  const sevBps = Math.round(clamp(severity, 0, 1) * 10000);
  const raw = notional
    .mul(new BN(coverRatioBps))
    .mul(new BN(sevBps))
    .div(new BN(10000))
    .div(new BN(10000));
  return BN.min(coverAmount, raw);
}
