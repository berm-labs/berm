import { describe, it, expect } from "vitest";
import BN from "bn.js";
import { RiskScorer } from "../src/risk";
import { CoverType } from "../src/constants";

const scorer = new RiskScorer();

describe("RiskScorer.score", () => {
  it("produces a score in [0, 100]", () => {
    const r = scorer.score({
      protocol: "Test",
      tvlUsd: 1_000_000,
      auditCount: 1,
      codeComplexity: 50,
      reputation: 50,
    });
    expect(r.score).toBeGreaterThanOrEqual(0);
    expect(r.score).toBeLessThanOrEqual(100);
  });

  it("scores a safer protocol lower than a riskier one", () => {
    const safe = scorer.score({
      protocol: "Safe",
      tvlUsd: 5_000_000_000,
      auditCount: 5,
      codeComplexity: 20,
      reputation: 95,
    });
    const risky = scorer.score({
      protocol: "Risky",
      tvlUsd: 500_000,
      auditCount: 0,
      codeComplexity: 90,
      reputation: 10,
    });
    expect(safe.score).toBeLessThan(risky.score);
    expect(safe.premiumRateBps).toBeLessThan(risky.premiumRateBps);
  });

  it("keeps the premium rate within the configured band", () => {
    const r = scorer.score({
      protocol: "Max",
      tvlUsd: 1,
      auditCount: 0,
      codeComplexity: 100,
      reputation: 0,
    });
    expect(r.premiumRateBps).toBeGreaterThanOrEqual(150);
    expect(r.premiumRateBps).toBeLessThanOrEqual(3500);
  });

  it("exposes the four risk components", () => {
    const r = scorer.score({
      protocol: "Comp",
      tvlUsd: 1_000_000,
      auditCount: 2,
      codeComplexity: 40,
      reputation: 60,
    });
    expect(r.components).toHaveProperty("tvl");
    expect(r.components).toHaveProperty("audits");
    expect(r.components).toHaveProperty("codeComplexity");
    expect(r.components).toHaveProperty("reputation");
  });
});

describe("RiskScorer.quote", () => {
  it("pro-rates the annual rate to the duration", () => {
    const risk = scorer.score({
      protocol: "Q",
      tvlUsd: 2_000_000_000,
      auditCount: 3,
      codeComplexity: 60,
      reputation: 80,
    });
    const full = scorer.quote(CoverType.Liquidation, new BN(10_000_000), 365, risk);
    const half = scorer.quote(CoverType.Liquidation, new BN(10_000_000), 30, risk);
    expect(half.premium.lt(full.premium)).toBe(true);
    expect(full.premiumRateBps).toBe(risk.premiumRateBps);
  });

  it("computes premium = coverAmount * rate * duration/365", () => {
    const risk = { protocol: "X", score: 0, components: { tvl: 0, audits: 0, codeComplexity: 0, reputation: 0 }, premiumRateBps: 1000 };
    // 1,000,000 * 0.10 * (365/365) = 100,000
    const q = scorer.quote(CoverType.Depeg, new BN(1_000_000), 365, risk);
    expect(q.premium.toString()).toBe("100000");
  });
});
