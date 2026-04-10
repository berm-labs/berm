import { describe, it, expect } from "vitest";
import BN from "bn.js";
import { backtest, SCENARIOS } from "../src/backtest";
import { CoverType } from "../src/constants";

describe("SCENARIOS", () => {
  it("exposes the three canonical incidents", () => {
    expect(Object.keys(SCENARIOS).sort()).toEqual([
      "mango2022",
      "msol2023",
      "usdc2023",
    ]);
  });

  it("derives severities from the public record", () => {
    expect(SCENARIOS.mango2022.severity).toBeCloseTo(0.9077, 3);
    expect(SCENARIOS.usdc2023.severity).toBeCloseTo(0.71, 6);
    expect(SCENARIOS.msol2023.severity).toBeCloseTo(0.021, 6);
  });
});

describe("backtest", () => {
  it("reproduces the USDC 2023 depeg payout", () => {
    const r = backtest("usdc2023", new BN(50000), 10000, new BN(50000));
    expect(r.payout.toString()).toBe("35500");
    expect(r.triggered).toBe(true);
    expect(r.coverType).toBe(CoverType.Depeg);
  });

  it("reproduces the Mango 2022 exploit payout", () => {
    const r = backtest("mango2022", new BN(10000), 10000, new BN(10000));
    // floor(10000 * 0.9077) = 9077
    expect(r.payout.toString()).toBe("9077");
    expect(r.coverType).toBe(CoverType.Exploit);
  });

  it("reproduces the mSOL 2023 slashing payout", () => {
    const r = backtest("msol2023", new BN(20000), 10000, new BN(20000));
    // floor(20000 * 0.021) = 420
    expect(r.payout.toString()).toBe("420");
    expect(r.coverType).toBe(CoverType.Slashing);
  });

  it("accepts an inline scenario", () => {
    const r = backtest(
      { key: "custom", name: "Custom", coverType: CoverType.Oracle, severity: 0.5 },
      new BN(1000),
      10000,
      new BN(1000)
    );
    expect(r.payout.toString()).toBe("500");
  });

  it("throws on an unknown scenario key", () => {
    // @ts-expect-error deliberately invalid key
    expect(() => backtest("nope", new BN(1), 10000, new BN(1))).toThrow();
  });
});
