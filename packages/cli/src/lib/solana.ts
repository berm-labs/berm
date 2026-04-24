import { Connection, LAMPORTS_PER_SOL, PublicKey } from "@solana/web3.js";
import { TOKEN_PROGRAM_ID, TOKEN_2022_PROGRAM_ID } from "@solana/spl-token";
import { KNOWN_MINT_BY_ID } from "./constants.js";
import type { RuntimeConfig } from "./config.js";

export interface TokenHolding {
  mint: string;
  amount: number; // ui amount
  symbol: string | null;
  class: "stable" | "lst" | "governance" | "native" | "other";
}

export interface WalletSnapshot {
  owner: string;
  solBalance: number;
  holdings: TokenHolding[];
  slot: number;
}

export function parseWallet(address: string): PublicKey {
  try {
    return new PublicKey(address.trim());
  } catch {
    throw new Error(`Invalid wallet address: ${address}`);
  }
}

// Reads live on-chain state for a wallet: native SOL balance plus every SPL and
// Token-2022 account, classified against the known-mint registry. This is real
// data straight from the configured RPC; nothing here is mocked.
export async function readWallet(
  config: RuntimeConfig,
  address: string,
): Promise<WalletSnapshot> {
  const connection = new Connection(config.rpcUrl, "confirmed");
  const owner = parseWallet(address);

  const [lamports, slot, legacy, token2022] = await Promise.all([
    connection.getBalance(owner),
    connection.getSlot(),
    connection.getParsedTokenAccountsByOwner(owner, { programId: TOKEN_PROGRAM_ID }),
    connection.getParsedTokenAccountsByOwner(owner, { programId: TOKEN_2022_PROGRAM_ID }),
  ]);

  const holdings: TokenHolding[] = [];
  for (const { account } of [...legacy.value, ...token2022.value]) {
    const info = account.data.parsed?.info;
    if (!info) continue;
    const uiAmount: number = info.tokenAmount?.uiAmount ?? 0;
    if (uiAmount <= 0) continue;
    const mint: string = info.mint;
    const known = KNOWN_MINT_BY_ID.get(mint);
    holdings.push({
      mint,
      amount: uiAmount,
      symbol: known?.symbol ?? null,
      class: known?.class ?? "other",
    });
  }

  holdings.sort((a, b) => b.amount - a.amount);

  return {
    owner: owner.toBase58(),
    solBalance: lamports / LAMPORTS_PER_SOL,
    holdings,
    slot,
  };
}
