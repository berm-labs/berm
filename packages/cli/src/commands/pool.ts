import ora from "ora";
import { resolveConfig, type GlobalFlags } from "../lib/config.js";
import { BermApi, type PoolSummary } from "../lib/api.js";
import { theme } from "../lib/theme.js";
import { heading, num, pct, table, usd } from "../lib/format.js";

export async function poolListCommand(opts: GlobalFlags): Promise<void> {
  const config = resolveConfig(opts);
  const api = new BermApi(config);
  const spinner = config.json ? null : ora({ text: "Loading cover pools...", color: "cyan" }).start();

  let pools: PoolSummary[];
  try {
    pools = await api.listPools();
  } catch (err) {
    spinner?.fail("Pool list failed.");
    throw err;
  }
  spinner?.succeed(`Loaded ${pools.length} cover pools.`);

  if (config.json) {
    process.stdout.write(JSON.stringify(pools, null, 2) + "\n");
    return;
  }

  console.log(heading("Active cover pools"));
  console.log(
    table(
      ["Pool", "Cover", "TVL", "Used", "APR", "Covers", "Triggers"],
      pools.map((p) => [
        theme.value(p.id),
        theme.accent(p.label),
        usd(p.tvlUsd),
        pct(p.utilization),
        pct(p.premiumApr),
        num(p.activeCovers, 0),
        num(p.triggeredEvents, 0),
      ]),
    ),
  );

  const totalTvl = pools.reduce((a, p) => a + p.tvlUsd, 0);
  const totalCovers = pools.reduce((a, p) => a + p.activeCovers, 0);
  console.log(
    "\n  " +
      theme.muted(`Total TVL ${usd(totalTvl)} across ${pools.length} pools, ${num(totalCovers, 0)} active covers.`),
  );
}
