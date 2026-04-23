import ora from "ora";
import { Connection, PublicKey } from "@solana/web3.js";
import { resolveConfig, type GlobalFlags } from "../lib/config.js";
import { BermApi, ApiError } from "../lib/api.js";
import {
  COVER_TYPE_META,
  explorerTxUrl,
  parseCoverType,
  parseDurationDays,
} from "../lib/constants.js";
import { estimatePremium, scoreWallet } from "../lib/risk.js";
import { readWallet, parseWallet } from "../lib/solana.js";
import {
  buildCoverTransaction,
  loadKeypair,
  submitTransaction,
  InsufficientFundsError,
  KeypairError,
  SimulationError,
  SendError,
  type CoverRequest,
} from "../lib/transaction.js";
import { theme } from "../lib/theme.js";
import { heading, kv, num, usd } from "../lib/format.js";

interface CoverFlags extends GlobalFlags {
  type?: string;
  amount?: string;
  duration?: string;
  wallet?: string;
  keypair?: string;
  send?: boolean;
}

export async function coverCommand(opts: CoverFlags): Promise<void> {
  const config = resolveConfig(opts);

  if (!opts.type) throw new Error("Missing --type <exploit|depeg|slashing|liquidation|oracle>.");
  if (!opts.amount) throw new Error("Missing --amount <usd>.");

  const coverType = parseCoverType(opts.type);
  const amountUsd = Number(opts.amount);
  if (!Number.isFinite(amountUsd) || amountUsd <= 0) {
    throw new Error(`Invalid --amount "${opts.amount}". Provide a positive USD number.`);
  }
  const durationDays = parseDurationDays(opts.duration ?? "30d");
  const meta = COVER_TYPE_META[coverType];

  if (opts.send && !opts.keypair) {
    throw new Error("--send requires --keypair <path> to sign the transaction.");
  }

  const spinner = config.json ? null : ora({ text: "Pricing cover...", color: "cyan" }).start();

  // Load the signer first (if provided) so the fee payer is known.
  let keypair = null;
  let feePayer: PublicKey;
  if (opts.keypair) {
    try {
      keypair = await loadKeypair(opts.keypair);
    } catch (err) {
      spinner?.fail("Keypair load failed.");
      throw err;
    }
    feePayer = keypair.publicKey;
  } else if (opts.wallet) {
    feePayer = parseWallet(opts.wallet);
  } else {
    spinner?.fail("No fee payer.");
    throw new Error("Provide --wallet <address> or --keypair <path> to build the transaction.");
  }

  // Risk-adjust the local estimate from the wallet's live exposure.
  let riskScore: number | undefined;
  const riskWallet = opts.wallet ?? feePayer.toBase58();
  try {
    const snapshot = await readWallet(config, riskWallet);
    riskScore = scoreWallet(snapshot).score;
  } catch {
    riskScore = undefined;
  }

  // Authoritative backend quote, with a labelled local fallback.
  const api = new BermApi(config);
  let source: "backend" | "local";
  let premiumUsd: number;
  let rateBps: number;
  try {
    const quote = await api.coverQuote({ coverType, amountUsd, durationDays, wallet: riskWallet });
    source = "backend";
    premiumUsd = quote.premiumUsd;
    rateBps = quote.rateBps;
  } catch (err) {
    if (!(err instanceof ApiError)) throw err;
    const local = estimatePremium({ coverType, amountUsd, durationDays, riskScore });
    source = "local";
    premiumUsd = local.premiumUsd;
    rateBps = local.rateBps;
  }

  // Build the real cover-intent transaction against the live cluster.
  const connection = new Connection(config.rpcUrl, "confirmed");
  const request: CoverRequest = {
    coverType,
    amountUsd,
    durationDays,
    premiumUsd,
    rateBps,
    wallet: riskWallet,
  };

  let tx;
  try {
    tx = await buildCoverTransaction(connection, feePayer, request);
  } catch (err) {
    spinner?.fail("Transaction build failed.");
    throw new Error(`Could not build transaction: ${err instanceof Error ? err.message : String(err)}`);
  }

  spinner?.succeed(source === "backend" ? "Quote retrieved." : "Quote estimated offline.");

  // Submit or serialize.
  let signature: string | null = null;
  let serialized: string | null = null;
  if (opts.send && keypair) {
    const sendSpinner = config.json ? null : ora({ text: "Submitting transaction...", color: "cyan" }).start();
    try {
      const result = await submitTransaction(connection, tx, keypair);
      signature = result.signature;
      sendSpinner?.succeed("Transaction confirmed.");
    } catch (err) {
      sendSpinner?.fail("Transaction failed.");
      reportSubmitError(err);
      process.exitCode = 1;
      return;
    }
  } else {
    serialized = tx
      .serialize({ requireAllSignatures: false, verifySignatures: false })
      .toString("base64");
  }

  if (config.json) {
    process.stdout.write(
      JSON.stringify(
        {
          cluster: config.cluster,
          coverType,
          amountUsd,
          durationDays,
          premiumUsd,
          rateBps,
          source,
          feePayer: feePayer.toBase58(),
          signature,
          unsignedTransactionBase64: serialized,
          explorer: signature ? explorerTxUrl(signature, config.cluster) : null,
        },
        null,
        2,
      ) + "\n",
    );
    return;
  }

  console.log(heading("Cover quote"));
  console.log(kv("Cluster", config.cluster));
  console.log(kv("Cover type", theme.accent(meta.label)));
  console.log(kv("Trigger", meta.trigger));
  console.log(kv("Oracle", meta.oracle));
  console.log(kv("Cover amount", usd(amountUsd)));
  console.log(kv("Duration", `${durationDays} days`));
  console.log(kv("Premium", theme.token(usd(premiumUsd))));
  console.log(kv("Rate", `${num(rateBps, 0)} bps`));
  if (riskScore !== undefined) console.log(kv("Wallet risk", `${riskScore} / 100`));
  console.log(kv("Fee payer", feePayer.toBase58()));
  console.log(kv("Quote source", source === "backend" ? theme.ok("backend") : theme.warn("local estimate")));

  if (signature) {
    console.log(heading("Submitted"));
    console.log(kv("Signature", theme.ok(signature)));
    console.log(kv("Explorer", theme.value(explorerTxUrl(signature, config.cluster))));
  } else {
    console.log(heading("Unsigned transaction"));
    console.log(
      "  " +
        theme.muted(
          "Cover-intent transaction built against the live cluster. Sign it in a wallet, or re-run with --keypair <path> --send to submit.",
        ),
    );
    console.log("\n" + theme.value(serialized ?? ""));
  }

  if (source === "local") {
    console.log(
      "\n  " +
        theme.muted(
          "Premium is a local estimate (backend unreachable). Set BERM_API_URL for an authoritative quote.",
        ),
    );
  }
}

function reportSubmitError(err: unknown): void {
  if (err instanceof KeypairError) {
    process.stderr.write(theme.danger(`\nkeypair error: ${err.message}\n`));
  } else if (err instanceof InsufficientFundsError) {
    process.stderr.write(theme.danger(`\ninsufficient funds: ${err.message}\n`));
  } else if (err instanceof SimulationError) {
    process.stderr.write(theme.danger(`\nsimulation error: ${err.message}\n`));
    for (const line of err.logs) process.stderr.write(theme.muted(`  ${line}\n`));
  } else if (err instanceof SendError) {
    process.stderr.write(theme.danger(`\nsend error: ${err.message}\n`));
  } else {
    process.stderr.write(theme.danger(`\nerror: ${err instanceof Error ? err.message : String(err)}\n`));
  }
}
