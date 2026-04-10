import { PublicKey } from "@solana/web3.js";
import BN from "bn.js";
import { CoverType } from "./constants";

/** Lifecycle state of a claim against a cover position. */
export enum ClaimStatus {
  None = 0,
  Proposed = 1,
  Disputed = 2,
  Settled = 3,
  Rejected = 4,
}

/** On-chain cover pool account, one per cover type per covered asset. */
export interface CoverPoolAccount {
  address: PublicKey;
  coverType: CoverType;
  coveredAsset: PublicKey;
  /** Total capital underwritten by liquidity providers, base units. */
  totalCapital: BN;
  /** Aggregate outstanding cover sold against the pool, base units. */
  totalCoverOutstanding: BN;
  /** Accrued premium pending distribution, base units. */
  premiumAccrued: BN;
  /** Number of currently active cover positions. */
  activePositions: number;
  /** Parametric trigger threshold in basis points. */
  thresholdBps: number;
  /** Persistence window in slots. */
  windowSlots: number;
  bump: number;
}

/** On-chain cover position account owned by a buyer. */
export interface CoverPositionAccount {
  address: PublicKey;
  owner: PublicKey;
  pool: PublicKey;
  coverType: CoverType;
  /** Maximum payout (C), base units. */
  coverAmount: BN;
  /** Premium paid (P), base units. */
  premiumPaid: BN;
  /** Cover ratio in basis points (e.g. 10000 = 100%). */
  coverRatioBps: number;
  /** Slot at which the position becomes active. */
  startSlot: BN;
  /** Slot at which the position expires. */
  expirySlot: BN;
  claimStatus: ClaimStatus;
  bump: number;
}

/** Resolution record produced by the claim resolver. */
export interface ClaimRecord {
  address: PublicKey;
  position: PublicKey;
  status: ClaimStatus;
  /** Severity ratio in basis points used to compute payout. */
  severityBps: number;
  /** Settled payout amount, base units. */
  payout: BN;
  /** Slot at which the trigger condition was confirmed. */
  triggerSlot: BN;
}

/** Normalized observation from a single oracle source. */
export interface OracleObservation {
  price: number;
  /** Confidence interval around the price (Pyth). */
  confidence: number;
  /** Publish slot or timestamp of the observation. */
  publishSlot: number;
  source: "pyth" | "switchboard";
}

/** Aggregated dual-oracle reading with a divergence measure. */
export interface AggregatedPrice {
  pyth: OracleObservation;
  switchboard?: OracleObservation;
  /** Mid price used for downstream math. */
  mid: number;
  /** Absolute divergence as a fraction of mid price. */
  divergence: number;
  /** True when divergence exceeds the OracleCover threshold. */
  diverged: boolean;
}

/** Protocol risk assessment produced by the underwriter scorer. */
export interface RiskScore {
  protocol: string;
  /** Composite risk score, 0 (safe) to 100 (high risk). */
  score: number;
  components: {
    tvl: number;
    audits: number;
    codeComplexity: number;
    reputation: number;
  };
  /** Recommended annualized premium rate in basis points. */
  premiumRateBps: number;
}

/** Quote for a prospective cover purchase. */
export interface CoverQuote {
  coverType: CoverType;
  coverAmount: BN;
  durationDays: number;
  premium: BN;
  premiumRateBps: number;
}

/** Result of a single backtest replay. */
export interface BacktestResult {
  coverType: CoverType;
  scenario: string;
  severity: number;
  payout: BN;
  triggered: boolean;
}
