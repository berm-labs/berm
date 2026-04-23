import ora from "ora";
import { resolveConfig, type GlobalFlags } from "../lib/config.js";
import { BermApi, type OracleStatus } from "../lib/api.js";
import { theme } from "../lib/theme.js";
import { heading, num, table } from "../lib/format.js";

export async function oracleStatusCommand(opts: GlobalFlags): Promise<void> {
  const config = resolveConfig(opts);
  const api = new BermApi(config);
  const spinner = config.json ? null : ora({ text: "Querying oracle feeds...", color: "cyan" }).start();

  let feeds: OracleStatus[];
  try {
    feeds = await api.oracleStatus();
  } catch (err) {
    spinner?.fail("Oracle status failed.");
    throw err;
  }
  const unhealthy = feeds.filter((f) => !f.healthy).length;
  if (unhealthy > 0) spinner?.warn(`${unhealthy} of ${feeds.length} feeds degraded.`);
  else spinner?.succeed(`${feeds.length} feeds healthy.`);

  if (config.json) {
    process.stdout.write(JSON.stringify(feeds, null, 2) + "\n");
    return;
  }

  console.log(heading("Dual oracle status (Pyth + Switchboard)"));
  console.log(
    table(
      ["Feed", "Source", "Symbol", "Price", "Conf", "Stale", "Health"],
      feeds.map((f) => [
        theme.value(f.feed),
        theme.accent(f.source),
        f.symbol,
        `$${num(f.price, 4)}`,
        `${num(f.confidence, 4)}`,
        `${num(f.staleSlots, 0)} sl`,
        f.healthy ? theme.ok("ok") : theme.danger("degraded"),
      ]),
    ),
  );
}
