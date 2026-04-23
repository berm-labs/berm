import ora from "ora";
import { resolveConfig, type GlobalFlags } from "../lib/config.js";
import { BermApi, type ClaimStatus } from "../lib/api.js";
import { theme } from "../lib/theme.js";
import { heading, kv, usd } from "../lib/format.js";

interface ClaimFlags extends GlobalFlags {
  id?: string;
}

export async function claimCommand(opts: ClaimFlags): Promise<void> {
  const config = resolveConfig(opts);
  if (!opts.id) throw new Error("Missing --id <claim-id>.");

  const api = new BermApi(config);
  const spinner = config.json ? null : ora({ text: "Resolving claim...", color: "cyan" }).start();

  let status: ClaimStatus;
  try {
    status = await api.claim(opts.id);
  } catch (err) {
    spinner?.fail("Claim lookup failed.");
    throw err;
  }
  spinner?.succeed("Claim resolved.");

  if (config.json) {
    process.stdout.write(JSON.stringify(status, null, 2) + "\n");
    return;
  }

  console.log(heading("Claim status"));
  console.log(kv("Claim ID", status.id));
  console.log(kv("Cover type", theme.accent(status.coverType)));
  console.log(kv("State", stateCell(status.state)));
  console.log(kv("Triggered", status.triggeredAt ?? theme.muted("not triggered")));
  console.log(kv("Payout", status.payoutUsd !== null ? theme.token(usd(status.payoutUsd)) : theme.muted("pending")));
  console.log(kv("Detail", status.reason));
}

function stateCell(state: ClaimStatus["state"]): string {
  switch (state) {
    case "paid":
      return theme.token("paid");
    case "triggered":
      return theme.safe("triggered");
    case "monitoring":
      return theme.ok("monitoring");
    case "disputed":
      return theme.warn("disputed");
    case "rejected":
      return theme.danger("rejected");
    default:
      return theme.value(state);
  }
}
