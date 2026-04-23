import type { RuntimeConfig } from "./config.js";

export class ApiError extends Error {
  readonly status: number;
  readonly endpoint: string;
  constructor(message: string, status: number, endpoint: string) {
    super(message);
    this.name = "ApiError";
    this.status = status;
    this.endpoint = endpoint;
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

export interface ClaimStatus {
  id: string;
  coverType: string;
  state: "monitoring" | "triggered" | "paid" | "disputed" | "rejected";
  triggeredAt: string | null;
  payoutUsd: number | null;
  reason: string;
}

export interface CoverQuoteResponse {
  coverType: string;
  amountUsd: number;
  durationDays: number;
  premiumUsd: number;
  rateBps: number;
  poolId: string;
  poolUtilization: number;
}

// Thin typed HTTP client over the Berm backend. Every method performs a real
// request; failures surface as ApiError with the endpoint and status so the
// command layer can render an honest error instead of fabricated data.
export class BermApi {
  private readonly base: string;

  constructor(config: RuntimeConfig) {
    this.base = config.apiUrl;
  }

  get baseUrl(): string {
    return this.base;
  }

  private async get<T>(path: string, params?: Record<string, string>): Promise<T> {
    const url = new URL(this.base + path);
    if (params) {
      for (const [k, v] of Object.entries(params)) url.searchParams.set(k, v);
    }
    return this.request<T>(url.toString(), { method: "GET" });
  }

  private async post<T>(path: string, body: unknown): Promise<T> {
    return this.request<T>(this.base + path, {
      method: "POST",
      headers: { "content-type": "application/json" },
      body: JSON.stringify(body),
    });
  }

  private async request<T>(url: string, init: RequestInit): Promise<T> {
    const controller = new AbortController();
    const timer = setTimeout(() => controller.abort(), 15_000);
    let res: Response;
    try {
      res = await fetch(url, { ...init, signal: controller.signal });
    } catch (err) {
      const reason = err instanceof Error ? err.message : String(err);
      throw new ApiError(`request failed: ${reason}`, 0, url);
    } finally {
      clearTimeout(timer);
    }
    if (!res.ok) {
      const text = await res.text().catch(() => "");
      throw new ApiError(
        text || `${res.status} ${res.statusText}`,
        res.status,
        url,
      );
    }
    return (await res.json()) as T;
  }

  listPools(): Promise<PoolSummary[]> {
    return this.get<PoolSummary[]>("/pool/list");
  }

  oracleStatus(): Promise<OracleStatus[]> {
    return this.get<OracleStatus[]>("/oracle/status");
  }

  claim(id: string): Promise<ClaimStatus> {
    return this.get<ClaimStatus>(`/claim/${encodeURIComponent(id)}`);
  }

  coverQuote(input: {
    coverType: string;
    amountUsd: number;
    durationDays: number;
    wallet?: string;
  }): Promise<CoverQuoteResponse> {
    return this.post<CoverQuoteResponse>("/cover/quote", input);
  }
}
