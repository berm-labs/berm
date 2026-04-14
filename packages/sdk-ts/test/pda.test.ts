import { describe, it, expect } from "vitest";
import { PublicKey } from "@solana/web3.js";
import {
  coverPoolPda,
  poolVaultPda,
  coverPositionPda,
  claimPda,
  underwriterPda,
} from "../src/pda";
import { BERM_PROGRAM_ID, CoverType } from "../src/constants";

const USDC = new PublicKey("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");
const OWNER = new PublicKey("9xQeWvG816bUx9EPjHmaT23yvVM2ZWbrrpZb9PusVFin");

describe("coverPoolPda", () => {
  it("is deterministic for the same inputs", () => {
    const [a] = coverPoolPda(CoverType.Depeg, USDC);
    const [b] = coverPoolPda(CoverType.Depeg, USDC);
    expect(a.toBase58()).toBe(b.toBase58());
  });
  it("differs by cover type", () => {
    const [depeg] = coverPoolPda(CoverType.Depeg, USDC);
    const [exploit] = coverPoolPda(CoverType.Exploit, USDC);
    expect(depeg.toBase58()).not.toBe(exploit.toBase58());
  });
  it("is owned by the program (on curve off)", () => {
    const [pool, bump] = coverPoolPda(CoverType.Depeg, USDC);
    expect(PublicKey.isOnCurve(pool.toBytes())).toBe(false);
    expect(bump).toBeGreaterThanOrEqual(0);
    expect(bump).toBeLessThanOrEqual(255);
  });
  it("respects a program id override", () => {
    const alt = new PublicKey("Vote111111111111111111111111111111111111111");
    const [def] = coverPoolPda(CoverType.Depeg, USDC, BERM_PROGRAM_ID);
    const [over] = coverPoolPda(CoverType.Depeg, USDC, alt);
    expect(def.toBase58()).not.toBe(over.toBase58());
  });
});

describe("coverPositionPda", () => {
  it("differs by index", () => {
    const [pool] = coverPoolPda(CoverType.Depeg, USDC);
    const [p0] = coverPositionPda(pool, OWNER, 0);
    const [p1] = coverPositionPda(pool, OWNER, 1);
    expect(p0.toBase58()).not.toBe(p1.toBase58());
  });
  it("differs by owner", () => {
    const [pool] = coverPoolPda(CoverType.Depeg, USDC);
    const [a] = coverPositionPda(pool, OWNER, 0);
    const [b] = coverPositionPda(pool, USDC, 0);
    expect(a.toBase58()).not.toBe(b.toBase58());
  });
});

describe("derived child PDAs", () => {
  it("derives a vault, claim, and underwriter address", () => {
    const [pool] = coverPoolPda(CoverType.Oracle, USDC);
    const [position] = coverPositionPda(pool, OWNER, 0);
    expect(poolVaultPda(pool)[0]).toBeInstanceOf(PublicKey);
    expect(claimPda(position)[0]).toBeInstanceOf(PublicKey);
    expect(underwriterPda(OWNER)[0]).toBeInstanceOf(PublicKey);
  });
});
