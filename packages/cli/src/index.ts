import { Command } from "commander";
import { createRequire } from "node:module";
import { fileURLToPath } from "node:url";
import { dirname, join } from "node:path";
import { theme, masthead } from "./lib/theme.js";
import { scanCommand } from "./commands/scan.js";
import { coverCommand } from "./commands/cover.js";
import { claimCommand } from "./commands/claim.js";
import { poolListCommand } from "./commands/pool.js";
import { oracleStatusCommand } from "./commands/oracle.js";

const require = createRequire(import.meta.url);
const pkg = require(join(dirname(fileURLToPath(import.meta.url)), "..", "package.json")) as {
  version: string;
};

function withGlobals(cmd: Command): Command {
  return cmd
    .option("--cluster <name>", "Solana cluster: devnet (default), mainnet, testnet")
    .option("--rpc <url>", "Solana RPC endpoint (public RPC only)")
    .option("--api <url>", "Berm backend base URL")
    .option("--json", "Emit machine-readable JSON", false);
}

async function run(action: () => Promise<void>): Promise<void> {
  try {
    await action();
  } catch (err) {
    const message = err instanceof Error ? err.message : String(err);
    process.stderr.write(theme.danger(`\nerror: ${message}\n`));
    process.exitCode = 1;
  }
}

const program = new Command();

program
  .name("berm")
  .description("Command line client for the Berm parametric DeFi cover protocol on Solana.")
  .version(pkg.version, "-v, --version", "Print the berm-cli version")
  .addHelpText("beforeAll", masthead(pkg.version) + "\n");

withGlobals(
  program
    .command("scan")
    .description("Scan a wallet position and print its parametric risk score"),
)
  .requiredOption("--wallet <address>", "Wallet address to scan")
  .action((opts) => run(() => scanCommand(opts)));

withGlobals(
  program
    .command("cover")
    .description("Price a cover for a given type, amount and duration"),
)
  .requiredOption("--type <type>", "exploit | depeg | slashing | liquidation | oracle")
  .requiredOption("--amount <usd>", "Cover amount in USD")
  .option("--duration <window>", "Cover window, e.g. 30d, 12w, 6m", "30d")
  .option("--wallet <address>", "Wallet to risk-adjust the premium and pay fees")
  .option("--keypair <path>", "Signer keypair file (Solana CLI format) to sign / send")
  .option("--send", "Sign and submit the transaction to the cluster", false)
  .action((opts) => run(() => coverCommand(opts)));

withGlobals(
  program
    .command("claim")
    .description("Check claim state and auto-trigger status"),
)
  .requiredOption("--id <claim-id>", "Claim identifier")
  .action((opts) => run(() => claimCommand(opts)));

const pool = program.command("pool").description("Inspect cover pools");
withGlobals(pool.command("list").description("List active cover pools")).action((opts) =>
  run(() => poolListCommand(opts)),
);

const oracle = program.command("oracle").description("Inspect oracle feeds");
withGlobals(oracle.command("status").description("Show Pyth + Switchboard feed health")).action(
  (opts) => run(() => oracleStatusCommand(opts)),
);

program.parseAsync(process.argv);
