import { PublicKey } from "@solana/web3.js";
import { TOKEN_2022_PROGRAM_ADDRESS } from "@solana-program/token-2022";

/**
 * The BERM cover pool executor program (the protocol's primary on-chain
 * identity). Deployed to Solana devnet. Override per environment through
 * {@link BermClientConfig.programId}.
 */
export const BERM_PROGRAM_ID = new PublicKey(
  "AMenBCW8sgtx2VriEYzdJkTCsUBF6FGQy8PhcNh9p7pH"
);

/** The Token-2022 cover pool vault program. Deployed to Solana devnet. */
export const POOL_VAULT_PROGRAM_ID = new PublicKey(
  "H4ifx5HYeHHvEuyJMdF1EpRSeNZJqRf3Vkhi4LT8N12T"
);

/** The claim resolver program. Deployed to Solana devnet. */
export const CLAIM_RESOLVER_PROGRAM_ID = new PublicKey(
  "GnS9Sii7PpELXQLyKwZRgrEpqma3GQwcSxtqNdCMmkk3"
);

/** Token-2022 program id, sourced from the canonical program package. */
export const TOKEN_2022_PROGRAM_ID = new PublicKey(
  TOKEN_2022_PROGRAM_ADDRESS as string
);

/** Public Solana devnet RPC. The BERM program is currently deployed to devnet. */
export const DEVNET_RPC_ENDPOINT = "https://api.devnet.solana.com";

/** Public Solana mainnet RPC, used once the program is promoted to mainnet. */
export const MAINNET_RPC_ENDPOINT = "https://api.mainnet-beta.solana.com";

/**
 * Default endpoint. Points at devnet to match the current deployment; switch
 * to {@link MAINNET_RPC_ENDPOINT} after the mainnet promotion. Only public,
 * keyless RPCs belong here -- never embed a keyed Helius/QuickNode URL.
 */
export const DEFAULT_RPC_ENDPOINT = DEVNET_RPC_ENDPOINT;

/** Pyth Hermes price service used for off-chain price reads. */
export const PYTH_HERMES_ENDPOINT = "https://hermes.pyth.network";

/** PDA seed prefixes. Mirror the on-chain seed scheme exactly. */
export const SEEDS = {
  coverPool: "cover_pool",
  coverPosition: "cover_position",
  poolVault: "pool_vault",
  claim: "claim",
  underwriter: "underwriter",
} as const;

/** The five parametric cover types supported by the protocol. */
export enum CoverType {
  Exploit = 0,
  Depeg = 1,
  Slashing = 2,
  Liquidation = 3,
  Oracle = 4,
}

/** Human-readable labels for each cover type. */
export const COVER_TYPE_LABEL: Record<CoverType, string> = {
  [CoverType.Exploit]: "ExploitCover",
  [CoverType.Depeg]: "DepegCover",
  [CoverType.Slashing]: "SlashingCover",
  [CoverType.Liquidation]: "LiquidationCover",
  [CoverType.Oracle]: "OracleCover",
};

/**
 * Default parametric thresholds and persistence windows per cover type,
 * as defined in docs/cover-spec.md. All values are governance-tunable
 * on-chain; these mirror the published defaults for client-side quoting
 * and backtesting.
 */
export interface CoverParams {
  /** Trigger threshold (ratio for exploit/oracle, band for depeg). */
  threshold: number;
  /** Consecutive slots a condition must persist before settlement. */
  window: number;
  /** Severity saturation cap where applicable. */
  severityCap: number;
}

export const DEFAULT_COVER_PARAMS: Record<CoverType, CoverParams> = {
  [CoverType.Exploit]: { threshold: 0.35, window: 2, severityCap: 1 },
  [CoverType.Depeg]: { threshold: 0.05, window: 8, severityCap: 0.1 },
  [CoverType.Slashing]: { threshold: 0, window: 1, severityCap: 1 },
  [CoverType.Liquidation]: { threshold: 0, window: 1, severityCap: 1 },
  [CoverType.Oracle]: { threshold: 0.01, window: 3, severityCap: 0.02 },
};
