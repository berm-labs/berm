import { readFile } from "node:fs/promises";
import { homedir } from "node:os";
import { resolve } from "node:path";
import {
  Connection,
  Keypair,
  PublicKey,
  Transaction,
  TransactionInstruction,
} from "@solana/web3.js";
import { BERM_PROGRAM_ID, MEMO_PROGRAM_ID, type CoverType } from "./constants.js";

// Distinct error types so the command layer can report each failure mode
// separately (network, signing, balance, simulation, send).
export class KeypairError extends Error {
  constructor(message: string) {
    super(message);
    this.name = "KeypairError";
  }
}
export class InsufficientFundsError extends Error {
  constructor(message: string) {
    super(message);
    this.name = "InsufficientFundsError";
  }
}
export class SimulationError extends Error {
  readonly logs: string[];
  constructor(message: string, logs: string[]) {
    super(message);
    this.name = "SimulationError";
    this.logs = logs;
  }
}
export class SendError extends Error {
  constructor(message: string) {
    super(message);
    this.name = "SendError";
  }
}

export interface CoverRequest {
  coverType: CoverType;
  amountUsd: number;
  durationDays: number;
  premiumUsd: number;
  rateBps: number;
  wallet: string;
}

// Canonical on-chain payload recorded with the cover-intent transaction.
export function coverMemo(request: CoverRequest): string {
  return JSON.stringify({
    p: "berm",
    v: 1,
    kind: "cover-intent",
    program: BERM_PROGRAM_ID,
    type: request.coverType,
    amountUsd: request.amountUsd,
    durationDays: request.durationDays,
    premiumUsd: request.premiumUsd,
    rateBps: request.rateBps,
    wallet: request.wallet,
  });
}

// Loads a Solana CLI keypair file (JSON array of 64 bytes).
export async function loadKeypair(path: string): Promise<Keypair> {
  const expanded = path.startsWith("~")
    ? resolve(homedir(), path.slice(1).replace(/^\/+/, ""))
    : resolve(path);
  let raw: string;
  try {
    raw = await readFile(expanded, "utf8");
  } catch {
    throw new KeypairError(`Keypair file not found or unreadable: ${expanded}`);
  }
  let bytes: number[];
  try {
    bytes = JSON.parse(raw) as number[];
  } catch {
    throw new KeypairError(`Keypair file is not valid JSON: ${expanded}`);
  }
  if (!Array.isArray(bytes) || (bytes.length !== 64 && bytes.length !== 32)) {
    throw new KeypairError("Keypair file must be a JSON array of 32 or 64 bytes.");
  }
  try {
    return Keypair.fromSecretKey(Uint8Array.from(bytes));
  } catch {
    throw new KeypairError("Keypair bytes are invalid.");
  }
}

// Builds a real, signable cover-intent transaction: a Memo instruction carrying
// the canonical cover request, with the fee payer set and a live blockhash from
// the configured RPC. Returns the unsigned transaction.
export async function buildCoverTransaction(
  connection: Connection,
  feePayer: PublicKey,
  request: CoverRequest,
): Promise<Transaction> {
  const memoIx = new TransactionInstruction({
    programId: new PublicKey(MEMO_PROGRAM_ID),
    keys: [{ pubkey: feePayer, isSigner: true, isWritable: false }],
    data: Buffer.from(coverMemo(request), "utf8"),
  });

  const tx = new Transaction().add(memoIx);
  tx.feePayer = feePayer;
  const { blockhash, lastValidBlockHeight } = await connection.getLatestBlockhash("confirmed");
  tx.recentBlockhash = blockhash;
  tx.lastValidBlockHeight = lastValidBlockHeight;
  return tx;
}

export interface SubmitResult {
  signature: string;
}

// Verifies the fee payer can cover the transaction fee, simulates the
// transaction against the live cluster, then signs, sends and confirms it.
export async function submitTransaction(
  connection: Connection,
  tx: Transaction,
  signer: Keypair,
): Promise<SubmitResult> {
  // 1. Balance / fee check against the live account.
  const fee = (await connection.getFeeForMessage(tx.compileMessage(), "confirmed")).value ?? 5000;
  const balance = await connection.getBalance(signer.publicKey, "confirmed");
  if (balance < fee) {
    throw new InsufficientFundsError(
      `Account ${signer.publicKey.toBase58()} has ${balance} lamports but needs at least ${fee} for fees.`,
    );
  }

  // 2. Real simulation; surface program logs on failure.
  tx.sign(signer);
  const sim = await connection.simulateTransaction(tx);
  if (sim.value.err) {
    throw new SimulationError(
      `Simulation failed: ${JSON.stringify(sim.value.err)}`,
      sim.value.logs ?? [],
    );
  }

  // 3. Send and confirm.
  let signature: string;
  try {
    signature = await connection.sendRawTransaction(tx.serialize(), {
      skipPreflight: false,
      preflightCommitment: "confirmed",
    });
  } catch (err) {
    throw new SendError(err instanceof Error ? err.message : String(err));
  }

  const confirmation = await connection.confirmTransaction(
    {
      signature,
      blockhash: tx.recentBlockhash!,
      lastValidBlockHeight: tx.lastValidBlockHeight!,
    },
    "confirmed",
  );
  if (confirmation.value.err) {
    throw new SendError(`Transaction ${signature} failed: ${JSON.stringify(confirmation.value.err)}`);
  }

  return { signature };
}
