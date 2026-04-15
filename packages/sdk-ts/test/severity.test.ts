import { describe, it, expect } from "vitest";
import BN from "bn.js";
import {
  clamp,
  exploitSeverity,
  depegSeverity,
  slashingSeverity,
  liquidationSeverity,
  oracleSeverity,
  payout,
} from "../src/severity";

describe("clamp", () => {
  it("bounds to range", () => {
    expect(clamp(-1, 0, 1)).toBe(0);
    expect(clamp(2, 0, 1)).toBe(1);
    expect(clamp(0.5, 0, 1)).toBe(0.5);
  });
});

describe("exploitSeverity", () => {
  it("is zero at or below the threshold", () => {
    expect(exploitSeverity(0.35)).toBe(0);
    expect(exploitSeverity(0.1)).toBe(0);
  });
  it("normalizes drop above the threshold", () => {
    // (0.94 - 0.35) / (1 - 0.35) = 0.9077...
    expect(exploitSeverity(0.94)).toBeCloseTo(0.9077, 3);
  });
  it("saturates at a full drain", () => {
    expect(exploitSeverity(1)).toBe(1);
  });
  it("honors a custom threshold", () => {
    expect(exploitSeverity(0.5, 0.5)).toBe(0);
    expect(exploitSeverity(0.75, 0.5)).toBeCloseTo(0.5, 6);
  });
});

describe("depegSeverity", () => {
  it("is zero inside the band", () => {
    expect(depegSeverity(1.0)).toBe(0);
    expect(depegSeverity(0.96)).toBe(0);
    expect(depegSeverity(1.04)).toBe(0);
  });
  it("scales depth below the band", () => {
    // depth = 0.95 - 0.879 = 0.071; /0.10 = 0.71
    expect(depegSeverity(0.879)).toBeCloseTo(0.71, 6);
  });
  it("scales depth above the band", () => {
    expect(depegSeverity(1.1)).toBeCloseTo(0.5, 6);
  });
  it("saturates past the cap", () => {
    expect(depegSeverity(0.5)).toBe(1);
  });
});

describe("slashingSeverity", () => {
  it("is the slashed proportion", () => {
    expect(slashingSeverity(2.1, 100)).toBeCloseTo(0.021, 6);
  });
  it("guards against zero total", () => {
    expect(slashingSeverity(5, 0)).toBe(0);
  });
  it("never exceeds one", () => {
    expect(slashingSeverity(150, 100)).toBe(1);
  });
});

describe("liquidationSeverity", () => {
  it("is loss over collateral", () => {
    expect(liquidationSeverity(400, 1000)).toBeCloseTo(0.4, 6);
  });
  it("guards against zero collateral", () => {
    expect(liquidationSeverity(100, 0)).toBe(0);
  });
});

describe("oracleSeverity", () => {
  it("is zero at or below the threshold", () => {
    expect(oracleSeverity(0.01)).toBe(0);
    expect(oracleSeverity(0.005)).toBe(0);
  });
  it("normalizes divergence above the threshold", () => {
    // (0.02 - 0.01) / 0.01 = 1
    expect(oracleSeverity(0.02)).toBe(1);
    expect(oracleSeverity(0.015)).toBeCloseTo(0.5, 6);
  });
});

describe("payout", () => {
  it("computes r * severity * notional", () => {
    // 1.0 * 0.71 * 50000 = 35500
    const p = payout(new BN(50000), 10000, 0.71, new BN(50000));
    expect(p.toString()).toBe("35500");
  });
  it("is bounded by the cover amount", () => {
    const p = payout(new BN(10000), 10000, 1, new BN(50000));
    expect(p.toString()).toBe("10000");
  });
  it("applies the cover ratio", () => {
    // 0.5 * 1.0 * 20000 = 10000
    const p = payout(new BN(1_000_000), 5000, 1, new BN(20000));
    expect(p.toString()).toBe("10000");
  });
  it("is zero when severity is zero", () => {
    const p = payout(new BN(50000), 10000, 0, new BN(50000));
    expect(p.toString()).toBe("0");
  });
  it("never returns negative and clamps severity above one", () => {
    const p = payout(new BN(1_000_000), 10000, 5, new BN(1000));
    expect(p.toString()).toBe("1000");
  });
});
