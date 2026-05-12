import { Connection, LAMPORTS_PER_SOL, PublicKey } from "@solana/web3.js";
import { config } from "./constants";

// Known mainnet mints used to classify holdings, mirroring the CLI registry.
const TOKEN_PROGRAM_ID = new PublicKey("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA");
const TOKEN_2022_PROGRAM_ID = new PublicKey("TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb");

const KNOWN_MINTS: Record<string, { symbol: string; class: "stable" | "lst" }> = {
  EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v: { symbol: "USDC", class: "stable" },
  Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB: { symbol: "USDT", class: "stable" },
  USDSwr9ApdHk5bvJKMjzff41FfuX8bSxdKcR81vTwcA: { symbol: "USDS", class: "stable" },
  "2b1kV6DkPAnxd5ixfnxCpjxmKwqjjaYmCZfHsFu24GXo": { symbol: "PYUSD", class: "stable" },
  mSoLzYCxHdYgdzU16g5QSh3i5K3z3KZK7ytfqcJm7So: { symbol: "mSOL", class: "lst" },
  J1toso1uCk3RLmjorhTtrVwY9HJ7X8V9yYac6Y7kGCPn: { symbol: "jitoSOL", class: "lst" },
  bSo13r4TkiE4KumL71LsHTPpL2euBYLFx6h9HP3piy1: { symbol: "bSOL", class: "lst" },
};

export interface RiskReport {
  owner: string;
  score: number;
  scores: { depeg: number; slashing: number; concentration: number };
  exposure: { stableTokens: number; lstTokens: number; nativeSol: number; otherTokens: number };
  slot: number;
}

function clamp(n: number, lo: number, hi: number): number {
  return Math.max(lo, Math.min(hi, n));
}

function saturate(value: number, half: number): number {
  if (value <= 0) return 0;
  return clamp((value / (value + half)) * 100, 0, 100);
}

function herfindahl(weights: number[]): number {
  const total = weights.reduce((a, b) => a + b, 0);
  if (total <= 0) return 0;
  return weights.reduce((acc, w) => acc + (w / total) ** 2, 0);
}

// Reads the wallet's live position from the configured public RPC and produces a
// parametric risk report. Identical scoring curve to the CLI so a wallet scores
// the same on phone and terminal.
export async function scoreWallet(address: string): Promise<RiskReport> {
  const connection = new Connection(config.rpcUrl, "confirmed");
  const owner = new PublicKey(address);

  const [lamports, slot, legacy, token2022] = await Promise.all([
    connection.getBalance(owner),
    connection.getSlot(),
    connection.getParsedTokenAccountsByOwner(owner, { programId: TOKEN_PROGRAM_ID }),
    connection.getParsedTokenAccountsByOwner(owner, { programId: TOKEN_2022_PROGRAM_ID }),
  ]);

  const exposure = {
    stableTokens: 0,
    lstTokens: 0,
    nativeSol: lamports / LAMPORTS_PER_SOL,
    otherTokens: 0,
  };

  for (const { account } of [...legacy.value, ...token2022.value]) {
    const info = (account.data as { parsed?: { info?: any } }).parsed?.info;
    if (!info) continue;
    const uiAmount: number = info.tokenAmount?.uiAmount ?? 0;
    if (uiAmount <= 0) continue;
    const known = KNOWN_MINTS[info.mint as string];
    if (known?.class === "stable") exposure.stableTokens += uiAmount;
    else if (known?.class === "lst") exposure.lstTokens += uiAmount;
    else exposure.otherTokens += uiAmount;
  }

  const depeg = saturate(exposure.stableTokens, 5_000);
  const slashing = saturate(exposure.lstTokens + exposure.nativeSol, 50);
  const concentrationIndex = herfindahl([
    exposure.stableTokens,
    exposure.lstTokens + exposure.nativeSol,
    exposure.otherTokens,
  ]);
  const concentration = clamp(concentrationIndex * 100, 0, 100);

  // Weighted by cover-type base rates (depeg 180, slashing 140, liquidation 260).
  const score = Math.round((depeg * 180 + slashing * 140 + concentration * 260) / 580);

  return {
    owner: owner.toBase58(),
    score,
    scores: { depeg, slashing, concentration },
    exposure,
    slot,
  };
}
