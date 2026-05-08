import { config } from "./constants";

export class ApiError extends Error {
  readonly status: number;
  constructor(message: string, status: number) {
    super(message);
    this.name = "ApiError";
    this.status = status;
  }
}

export interface PoolSummary {
  id: string;
  coverType: string;
  label: string;
  tvlUsd: number;
  utilization: number;
  premiumApr: number;
  activeCovers: number;
  triggeredEvents: number;
}

export interface OracleStatus {
  feed: string;
  source: string;
  symbol: string;
  price: number;
  confidence: number;
  staleSlots: number;
  healthy: boolean;
}

export interface CoverPosition {
  id: string;
  coverType: string;
  label: string;
  amountUsd: number;
  premiumUsd: number;
  startedAt: string;
  expiresAt: string;
  state: "active" | "triggered" | "expired";
  riskScore: number;
}

export interface AlertEvent {
  id: string;
  kind: "depeg" | "liquidation" | "claim" | "risk";
  title: string;
  body: string;
  coverType: string | null;
  createdAt: string;
  read: boolean;
}

export interface ProtocolStats {
  coversActive: number;
  tvlUsd: number;
  coverTypes: number;
  triggeredEvents: number;
}

async function request<T>(path: string, init?: RequestInit): Promise<T> {
  const controller = new AbortController();
  const timer = setTimeout(() => controller.abort(), 15_000);
  let res: Response;
  try {
    res = await fetch(config.apiUrl + path, {
      ...init,
      headers: { "content-type": "application/json", ...(init?.headers ?? {}) },
      signal: controller.signal,
    });
  } catch (err) {
    const reason = err instanceof Error ? err.message : String(err);
    throw new ApiError(`Network request failed: ${reason}`, 0);
  } finally {
    clearTimeout(timer);
  }
  if (!res.ok) {
    const text = await res.text().catch(() => "");
    throw new ApiError(text || `${res.status} ${res.statusText}`, res.status);
  }
  return (await res.json()) as T;
}

export const api = {
  stats: () => request<ProtocolStats>("/stats"),
  pools: () => request<PoolSummary[]>("/pool/list"),
  oracle: () => request<OracleStatus[]>("/oracle/status"),
  positions: (wallet: string) =>
    request<CoverPosition[]>(`/cover/positions?wallet=${encodeURIComponent(wallet)}`),
  alerts: (wallet: string) =>
    request<AlertEvent[]>(`/alerts?wallet=${encodeURIComponent(wallet)}`),
  registerPushToken: (body: { wallet: string; token: string; platform: string }) =>
    request<{ ok: boolean }>("/alerts/subscribe", { method: "POST", body: JSON.stringify(body) }),
};
