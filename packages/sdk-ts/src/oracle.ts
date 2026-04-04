import {
  PYTH_HERMES_ENDPOINT,
  DEFAULT_COVER_PARAMS,
  CoverType,
} from "./constants";
import { AggregatedPrice, OracleObservation } from "./types";

interface HermesPriceResponse {
  parsed?: Array<{
    id: string;
    price: { price: string; conf: string; expo: number; publish_time: number };
  }>;
}

/**
 * Reads prices from Pyth (via the Hermes price service) and aggregates them
 * with Switchboard observations, surfacing the cross-source divergence that
 * OracleCover triggers on.
 */
export class OracleAdapter {
  constructor(
    private readonly hermesEndpoint: string = PYTH_HERMES_ENDPOINT,
    private readonly oracleThreshold: number = DEFAULT_COVER_PARAMS[
      CoverType.Oracle
    ].threshold
  ) {}

  /**
   * Fetch the latest Pyth price for a feed id from Hermes.
   * @param feedId 32-byte hex price feed id (with or without 0x prefix)
   */
  async getPythPrice(feedId: string): Promise<OracleObservation> {
    const id = feedId.startsWith("0x") ? feedId.slice(2) : feedId;
    const url = `${this.hermesEndpoint}/v2/updates/price/latest?ids[]=${id}`;
    const res = await fetch(url);
    if (!res.ok) {
      throw new Error(`Hermes request failed: ${res.status} ${res.statusText}`);
    }
    const body = (await res.json()) as HermesPriceResponse;
    const parsed = body.parsed?.[0];
    if (!parsed) {
      throw new Error(`No price returned for feed ${id}`);
    }
    const expo = parsed.price.expo;
    const scale = Math.pow(10, expo);
    return {
      price: Number(parsed.price.price) * scale,
      confidence: Number(parsed.price.conf) * scale,
      publishSlot: parsed.price.publish_time,
      source: "pyth",
    };
  }

  /**
   * Aggregate a Pyth observation with an optional Switchboard observation,
   * computing the mid price and the divergence between the two sources.
   */
  aggregate(
    pyth: OracleObservation,
    switchboard?: OracleObservation
  ): AggregatedPrice {
    if (!switchboard) {
      return {
        pyth,
        mid: pyth.price,
        divergence: 0,
        diverged: false,
      };
    }
    const mid = (pyth.price + switchboard.price) / 2;
    const divergence = mid === 0 ? 0 : Math.abs(pyth.price - switchboard.price) / mid;
    return {
      pyth,
      switchboard,
      mid,
      divergence,
      diverged: divergence > this.oracleThreshold,
    };
  }

  /**
   * Convenience: fetch a Pyth price and aggregate it against a supplied
   * Switchboard observation in one call.
   */
  async getAggregated(
    feedId: string,
    switchboard?: OracleObservation
  ): Promise<AggregatedPrice> {
    const pyth = await this.getPythPrice(feedId);
    return this.aggregate(pyth, switchboard);
  }
}
