// Cover protocol primitives shared across commands.

export const COVER_TYPES = [
  "exploit",
  "depeg",
  "slashing",
  "liquidation",
  "oracle",
] as const;

export type CoverType = (typeof COVER_TYPES)[number];

export interface CoverTypeMeta {
  id: CoverType;
  label: string;
  trigger: string;
  oracle: string;
  // Base annualised rate in basis points used by the local quote estimator.
  baseRateBps: number;
}

export const COVER_TYPE_META: Record<CoverType, CoverTypeMeta> = {
  exploit: {
    id: "exploit",
    label: "ExploitCover",
    trigger: "TVL collapse + abnormal withdrawal pattern",
    oracle: "Pyth + on-chain TVL feed",
    baseRateBps: 320,
  },
  depeg: {
    id: "depeg",
    label: "DepegCover",
    trigger: "Stable price < 0.95 or > 1.05 sustained N slots",
    oracle: "Pyth + Switchboard",
    baseRateBps: 180,
  },
  slashing: {
    id: "slashing",
    label: "SlashingCover",
    trigger: "LST validator slashing epoch event",
    oracle: "Solana native stake feed",
    baseRateBps: 140,
  },
  liquidation: {
    id: "liquidation",
    label: "LiquidationCover",
    trigger: "Marginfi / Kamino / Solend liquidation event",
    oracle: "Per-protocol liquidation event + Pyth",
    baseRateBps: 260,
  },
  oracle: {
    id: "oracle",
    label: "OracleCover",
    trigger: "Pyth vs Switchboard divergence > N%",
    oracle: "Dual oracle divergence monitor",
    baseRateBps: 110,
  },
};

// Known mainnet mints used to classify wallet holdings for risk scoring.
// Each entry is real, verifiable on Solana Explorer.
export interface KnownMint {
  mint: string;
  symbol: string;
  decimals: number;
  class: "stable" | "lst" | "governance" | "native";
}

export const KNOWN_MINTS: KnownMint[] = [
  { mint: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v", symbol: "USDC", decimals: 6, class: "stable" },
  { mint: "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB", symbol: "USDT", decimals: 6, class: "stable" },
  { mint: "USDSwr9ApdHk5bvJKMjzff41FfuX8bSxdKcR81vTwcA", symbol: "USDS", decimals: 6, class: "stable" },
  { mint: "2b1kV6DkPAnxd5ixfnxCpjxmKwqjjaYmCZfHsFu24GXo", symbol: "PYUSD", decimals: 6, class: "stable" },
  { mint: "mSoLzYCxHdYgdzU16g5QSh3i5K3z3KZK7ytfqcJm7So", symbol: "mSOL", decimals: 9, class: "lst" },
  { mint: "J1toso1uCk3RLmjorhTtrVwY9HJ7X8V9yYac6Y7kGCPn", symbol: "jitoSOL", decimals: 9, class: "lst" },
  { mint: "bSo13r4TkiE4KumL71LsHTPpL2euBYLFx6h9HP3piy1", symbol: "bSOL", decimals: 9, class: "lst" },
  { mint: "jupSoLaHXQiZZTSfEWMTRRgpnyFm8f6sZdosWBjx93v", symbol: "JupSOL", decimals: 9, class: "lst" },
  { mint: "5oVNBeEEQvYi1cX3ir8Dx5n1P7pdxydbGF2X4TxVusJm", symbol: "INF", decimals: 9, class: "lst" },
];

export const KNOWN_MINT_BY_ID = new Map(KNOWN_MINTS.map((m) => [m.mint, m]));

export const CLUSTERS = ["devnet", "mainnet", "testnet"] as const;
export type Cluster = (typeof CLUSTERS)[number];

// Berm cover is live on Solana devnet; mainnet is pending deployment approval.
export const DEFAULT_CLUSTER: Cluster = "devnet";

export const CLUSTER_RPC: Record<Cluster, string> = {
  devnet: "https://api.devnet.solana.com",
  mainnet: "https://api.mainnet-beta.solana.com",
  testnet: "https://api.testnet.solana.com",
};

export const DEFAULT_RPC = CLUSTER_RPC[DEFAULT_CLUSTER];
export const DEFAULT_API = "https://api.berm.sh";

// Berm on-chain program addresses (deployed to devnet). Mainnet addresses are
// assigned at mainnet launch. These are program IDs, not the deploy/upgrade
// authority wallet -- cover-intent transactions tag the cover executor program.
export const BERM_PROGRAM_ID = "AMenBCW8sgtx2VriEYzdJkTCsUBF6FGQy8PhcNh9p7pH";
export const POOL_VAULT_PROGRAM_ID = "H4ifx5HYeHHvEuyJMdF1EpRSeNZJqRf3Vkhi4LT8N12T";
export const CLAIM_RESOLVER_PROGRAM_ID = "GnS9Sii7PpELXQLyKwZRgrEpqma3GQwcSxtqNdCMmkk3";

// SPL Memo program, deployed on every cluster.
export const MEMO_PROGRAM_ID = "MemoSq4gqABAXKb96qnH8TysNcWxMyWCqXgDLGmfcHr";

export function parseCluster(input: string): Cluster {
  const normalized = input.trim().toLowerCase();
  // Accept common aliases.
  const alias: Record<string, Cluster> = {
    "mainnet-beta": "mainnet",
    mainnet: "mainnet",
    devnet: "devnet",
    testnet: "testnet",
  };
  const cluster = alias[normalized];
  if (!cluster) {
    throw new Error(`Unknown cluster "${input}". Valid clusters: ${CLUSTERS.join(", ")}.`);
  }
  return cluster;
}

export function explorerTxUrl(signature: string, cluster: Cluster): string {
  const suffix = cluster === "mainnet" ? "" : `?cluster=${cluster}`;
  return `https://explorer.solana.com/tx/${signature}${suffix}`;
}

export function parseCoverType(input: string): CoverType {
  const normalized = input.trim().toLowerCase();
  if ((COVER_TYPES as readonly string[]).includes(normalized)) {
    return normalized as CoverType;
  }
  throw new Error(
    `Unknown cover type "${input}". Valid types: ${COVER_TYPES.join(", ")}.`,
  );
}

// Accepts forms like "30d", "12w", "6m", or a bare number of days.
export function parseDurationDays(input: string): number {
  const m = /^(\d+)\s*(d|w|m)?$/i.exec(input.trim());
  if (!m) {
    throw new Error(`Invalid duration "${input}". Use e.g. 30d, 12w, 6m.`);
  }
  const n = Number(m[1]);
  const unit = (m[2] ?? "d").toLowerCase();
  const days = unit === "w" ? n * 7 : unit === "m" ? n * 30 : n;
  if (days < 1 || days > 365) {
    throw new Error("Duration must resolve to between 1 and 365 days.");
  }
  return days;
}
