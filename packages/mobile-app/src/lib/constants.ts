import Constants from "expo-constants";

export const COVER_TYPES = ["exploit", "depeg", "slashing", "liquidation", "oracle"] as const;
export type CoverType = (typeof COVER_TYPES)[number];

export interface CoverTypeMeta {
  id: CoverType;
  label: string;
  short: string;
  trigger: string;
  oracle: string;
}

export const COVER_TYPE_META: Record<CoverType, CoverTypeMeta> = {
  exploit: {
    id: "exploit",
    label: "ExploitCover",
    short: "Exploit",
    trigger: "TVL collapse + abnormal withdrawals",
    oracle: "Pyth + on-chain TVL feed",
  },
  depeg: {
    id: "depeg",
    label: "DepegCover",
    short: "Depeg",
    trigger: "Stable price below 0.95 or above 1.05",
    oracle: "Pyth + Switchboard",
  },
  slashing: {
    id: "slashing",
    label: "SlashingCover",
    short: "Slashing",
    trigger: "LST validator slashing epoch event",
    oracle: "Solana native stake feed",
  },
  liquidation: {
    id: "liquidation",
    label: "LiquidationCover",
    short: "Liquidation",
    trigger: "Marginfi / Kamino / Solend liquidation",
    oracle: "Per-protocol event + Pyth",
  },
  oracle: {
    id: "oracle",
    label: "OracleCover",
    short: "Oracle",
    trigger: "Pyth vs Switchboard divergence",
    oracle: "Dual oracle monitor",
  },
};

export type AlertKind = "depeg" | "liquidation" | "claim" | "risk";

export interface AlertMeta {
  kind: AlertKind;
  label: string;
  description: string;
}

export const ALERT_META: Record<AlertKind, AlertMeta> = {
  depeg: { kind: "depeg", label: "Depeg detected", description: "A covered stable moved outside its peg band." },
  liquidation: { kind: "liquidation", label: "Liquidation imminent", description: "A monitored lending position nears its threshold." },
  claim: { kind: "claim", label: "Claim auto-triggered", description: "A parametric trigger fired and a payout was issued." },
  risk: { kind: "risk", label: "New risk detected", description: "A new risk vector was found in a covered position." },
};

interface AppExtra {
  apiUrl?: string;
  rpcUrl?: string;
  cluster?: string;
  siteUrl?: string;
}

const extra = (Constants.expoConfig?.extra ?? {}) as AppExtra;

// Berm cover is live on Solana devnet; mainnet is pending.
export const config = {
  apiUrl: (extra.apiUrl ?? "https://api.berm.sh").replace(/\/+$/, ""),
  rpcUrl: extra.rpcUrl ?? "https://api.devnet.solana.com",
  cluster: (extra.cluster ?? "devnet") as "mainnet-beta" | "devnet" | "testnet",
  siteUrl: extra.siteUrl ?? "https://berm.sh",
};

// Solana Mobile Wallet Adapter app identity.
export const APP_IDENTITY = {
  name: "BERM Alert",
  uri: config.siteUrl,
  icon: "favicon.ico",
};
