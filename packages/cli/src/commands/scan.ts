import ora from "ora";
import { resolveConfig, type GlobalFlags } from "../lib/config.js";
import { readWallet } from "../lib/solana.js";
import { scoreWallet } from "../lib/risk.js";
import { riskBand, riskColor, theme } from "../lib/theme.js";
import { heading, kv, meter, num, table } from "../lib/format.js";

interface ScanFlags extends GlobalFlags {
  wallet?: string;
}

export async function scanCommand(opts: ScanFlags): Promise<void> {
  const config = resolveConfig(opts);
  const wallet = opts.wallet;
  if (!wallet) {
    throw new Error("Missing --wallet <address>.");
  }

  const spinner = config.json
    ? null
    : ora({ text: "Reading on-chain position...", color: "cyan" }).start();

  let report;
  try {
    const snapshot = await readWallet(config, wallet);
    report = scoreWallet(snapshot);
  } catch (err) {
    spinner?.fail("Scan failed.");
    throw err;
  }
  spinner?.succeed("Position scanned.");

  if (config.json) {
    process.stdout.write(JSON.stringify(report, null, 2) + "\n");
    return;
  }

  const color = riskColor(report.score);
  console.log(heading("Wallet risk"));
  console.log(kv("Wallet", report.owner));
  console.log(kv("Cluster", config.cluster));
  console.log(kv("Slot", num(report.slot, 0)));
  console.log(
    kv(
      "Risk score",
      `${color(String(report.score).padStart(3))} / 100  ${color(`[${riskBand(report.score)}]`)}  ${theme.muted(meter(report.score))}`,
    ),
  );

  console.log(heading("Sub-scores"));
  console.log(
    table(
      ["Vector", "Score", "Gauge"],
      [
        ["Depeg", scoreCell(report.scores.depeg), theme.muted(meter(report.scores.depeg, 18))],
        ["Slashing", scoreCell(report.scores.slashing), theme.muted(meter(report.scores.slashing, 18))],
        ["Concentration", scoreCell(report.scores.concentration), theme.muted(meter(report.scores.concentration, 18))],
      ],
    ),
  );

  console.log(heading("Exposure"));
  console.log(kv("Stable units", num(report.exposure.stableTokens)));
  console.log(kv("LST tokens", num(report.exposure.lstTokens)));
  console.log(kv("Native SOL", num(report.exposure.nativeSol)));
  console.log(kv("Other tokens", num(report.exposure.otherTokens)));

  if (report.recommendations.length > 0) {
    console.log(heading("Recommended covers"));
    console.log(
      table(
        ["Cover", "Exposure", "Priority", "Why"],
        report.recommendations.map((r) => [
          theme.accent(r.label),
          `${num(r.exposureUnits)} ${r.exposureUnit}`,
          priorityCell(r.priority),
          theme.muted(r.rationale),
        ]),
      ),
    );
    console.log(
      "\n" +
        theme.muted("Design a cover with: ") +
        theme.value(`berm cover --type <type> --amount <usd> --duration 30d --wallet ${report.owner}`),
    );
  }

  if (report.notes.length > 0) {
    console.log(heading("Notes"));
    for (const note of report.notes) console.log("  " + theme.muted(note));
  }
}

function scoreCell(score: number): string {
  return riskColor(score)(String(Math.round(score)).padStart(3));
}

function priorityCell(p: "high" | "medium" | "low"): string {
  if (p === "high") return theme.danger("high");
  if (p === "medium") return theme.warn("medium");
  return theme.ok("low");
}
