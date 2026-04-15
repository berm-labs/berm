import { describe, it, expect } from "vitest";
import { OracleAdapter } from "../src/oracle";
import { OracleObservation } from "../src/types";

const adapter = new OracleAdapter();

function obs(price: number, source: "pyth" | "switchboard"): OracleObservation {
  return { price, confidence: 0.001, publishSlot: 1, source };
}

describe("OracleAdapter.aggregate", () => {
  it("reports no divergence with a single source", () => {
    const a = adapter.aggregate(obs(1.0, "pyth"));
    expect(a.divergence).toBe(0);
    expect(a.diverged).toBe(false);
    expect(a.mid).toBe(1.0);
  });

  it("computes the mid price across two sources", () => {
    const a = adapter.aggregate(obs(100, "pyth"), obs(102, "switchboard"));
    expect(a.mid).toBe(101);
  });

  it("computes divergence as a fraction of mid", () => {
    const a = adapter.aggregate(obs(100, "pyth"), obs(102, "switchboard"));
    // |100-102| / 101 = 0.0198
    expect(a.divergence).toBeCloseTo(0.0198, 4);
    expect(a.diverged).toBe(true); // > 1% default threshold
  });

  it("stays within tolerance for a tight spread", () => {
    const a = adapter.aggregate(obs(100, "pyth"), obs(100.2, "switchboard"));
    // 0.2 / 100.1 = 0.002 < 0.01
    expect(a.diverged).toBe(false);
  });

  it("honors a custom divergence threshold", () => {
    const strict = new OracleAdapter(undefined, 0.001);
    const a = strict.aggregate(obs(100, "pyth"), obs(100.2, "switchboard"));
    expect(a.diverged).toBe(true);
  });
});
