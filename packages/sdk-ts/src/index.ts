/**
 * @berm/sdk - TypeScript SDK for the BERM parametric DeFi cover protocol.
 *
 * Public surface: BermClient (entry point), CoverPool, CoverPosition,
 * ClaimResolver, RiskScorer, OracleAdapter, plus PDA helpers, severity math,
 * and historical backtests.
 */
export { BermClient } from "./client";
export type { BermClientConfig } from "./client";

export { CoverPool } from "./pool";
export { CoverPosition } from "./position";
export { ClaimResolver } from "./resolver";
export type { TriggerInput, TriggerEvaluation } from "./resolver";
export { RiskScorer } from "./risk";
export type { ProtocolSignals } from "./risk";
export { OracleAdapter } from "./oracle";

export {
  BERM_PROGRAM_ID,
  POOL_VAULT_PROGRAM_ID,
  CLAIM_RESOLVER_PROGRAM_ID,
  TOKEN_2022_PROGRAM_ID,
  DEFAULT_RPC_ENDPOINT,
  DEVNET_RPC_ENDPOINT,
  MAINNET_RPC_ENDPOINT,
  PYTH_HERMES_ENDPOINT,
  SEEDS,
  CoverType,
  COVER_TYPE_LABEL,
  DEFAULT_COVER_PARAMS,
} from "./constants";
export type { CoverParams } from "./constants";

export {
  coverPoolPda,
  poolVaultPda,
  coverPositionPda,
  claimPda,
  underwriterPda,
} from "./pda";

export {
  clamp,
  exploitSeverity,
  depegSeverity,
  slashingSeverity,
  liquidationSeverity,
  oracleSeverity,
  payout,
} from "./severity";

export { backtest, SCENARIOS } from "./backtest";
export type { Scenario } from "./backtest";

export { ClaimStatus } from "./types";
export type {
  CoverPoolAccount,
  CoverPositionAccount,
  ClaimRecord,
  OracleObservation,
  AggregatedPrice,
  RiskScore,
  CoverQuote,
  BacktestResult,
} from "./types";
