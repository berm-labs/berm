import {
  CLUSTER_RPC,
  DEFAULT_API,
  DEFAULT_CLUSTER,
  parseCluster,
  type Cluster,
} from "./constants.js";

export interface RuntimeConfig {
  cluster: Cluster;
  rpcUrl: string;
  apiUrl: string;
  json: boolean;
}

export interface GlobalFlags {
  cluster?: string;
  rpc?: string;
  api?: string;
  json?: boolean;
}

// Resolution order:
//   cluster: --cluster flag -> BERM_CLUSTER env -> devnet default
//   rpc:     --rpc flag -> BERM_RPC_URL / SOLANA_RPC_URL env -> cluster default
//   api:     --api flag -> BERM_API_URL / NEXT_PUBLIC_API_URL env -> public default
// Only public RPC endpoints belong here; secret RPC keys must never reach the client.
export function resolveConfig(flags: GlobalFlags): RuntimeConfig {
  const cluster = flags.cluster
    ? parseCluster(flags.cluster)
    : process.env.BERM_CLUSTER
      ? parseCluster(process.env.BERM_CLUSTER)
      : DEFAULT_CLUSTER;

  const rpcUrl =
    flags.rpc ??
    process.env.BERM_RPC_URL ??
    process.env.SOLANA_RPC_URL ??
    CLUSTER_RPC[cluster];

  const apiUrl = (
    flags.api ??
    process.env.BERM_API_URL ??
    process.env.NEXT_PUBLIC_API_URL ??
    DEFAULT_API
  ).replace(/\/+$/, "");

  return {
    cluster,
    rpcUrl,
    apiUrl,
    json: Boolean(flags.json),
  };
}
