# berm-cli

Command line client for **Berm**, the parametric DeFi cover protocol on Solana.
Scan wallet risk, price covers, track claims, and inspect cover pools and oracle
feeds from the terminal.

> Break the wave.

Berm provides parametric on-chain cover across five cover types: ExploitCover,
DepegCover, SlashingCover, LiquidationCover and OracleCover. Settlements are
deterministic functions of on-chain and oracle data -- no claim adjudication.
`berm-cli` reads live Solana state and talks to the Berm backend so you can
manage protection without leaving your shell.

> **Currently supports Solana devnet. Mainnet pending.** All commands default to
> the `devnet` cluster. Use `--cluster mainnet` once the mainnet program is live.

- Website: https://berm.sh
- X: https://x.com/berm_sh
- Source: https://github.com/berm-labs/berm

---

## Install

```bash
npm i -g berm-cli
```

Requires Node.js 20 or newer. Verify the install:

```bash
berm --version
berm --help
```

---

## Configuration

The CLI reads public configuration from flags or environment variables. No
secrets are ever required; only public RPC endpoints are used.

| Setting | Flag | Environment variable | Default |
| --- | --- | --- | --- |
| Cluster | `--cluster <name>` | `BERM_CLUSTER` | `devnet` |
| Solana RPC | `--rpc <url>` | `BERM_RPC_URL`, `SOLANA_RPC_URL` | cluster default (devnet: `https://api.devnet.solana.com`) |
| Backend API | `--api <url>` | `BERM_API_URL`, `NEXT_PUBLIC_API_URL` | `https://api.berm.sh` |
| JSON output | `--json` | — | off |

The cluster selects the default RPC unless `--rpc` / `BERM_RPC_URL` overrides it.
Only public RPC endpoints are used; no secret keys are read.

```bash
export BERM_CLUSTER="devnet"
export BERM_API_URL="https://api.berm.sh"
```

---

## Commands

### `berm scan`

Read a wallet's on-chain position and compute a parametric risk score. Native
SOL and every SPL / Token-2022 balance is classified into stable, LST and other
classes, then scored for depeg, slashing and concentration exposure.

```bash
berm scan --wallet 7xKXtg2CW87d97TXJSDpbD5jBkheTqA83TZRuJosgAsU
```

### `berm cover`

Price a cover and build the cover-intent transaction. When the backend is
reachable the authoritative quote is used; otherwise a local estimate is shown
and labelled as such. Pass `--wallet` to risk-adjust the premium from your live
exposure and set the fee payer.

The transaction is built against the live cluster: a real blockhash is fetched
and a Memo instruction carries the canonical cover request. Without `--keypair`
the unsigned transaction is serialized to base64 for signing in a wallet. With
`--keypair <path> --send` it is balance-checked, simulated, signed and submitted.

```bash
# Price + build an unsigned transaction (devnet)
berm cover --type depeg --amount 1000 --duration 30d --wallet 7xKX...gAsU

# Risk-adjusted quote for a larger position
berm cover --type liquidation --amount 25000 --duration 12w --wallet 7xKX...gAsU

# Sign and submit with a local keypair
berm cover --type depeg --amount 1000 --duration 30d --keypair ~/.config/solana/id.json --send
```

Errors are reported by category: keypair load, insufficient funds, simulation
(with program logs), and send / confirmation failures.

### `berm claim`

Check a claim's state and whether its oracle auto-trigger has fired.

```bash
berm claim --id clm_8f3a21
```

### `berm pool list`

List active cover pools with TVL, utilization, premium APR and trigger counts.

```bash
berm pool list
berm pool list --json
```

### `berm oracle status`

Show the health of the dual Pyth + Switchboard oracle feeds that drive
parametric triggers.

```bash
berm oracle status
```

### `berm version` / `berm help`

```bash
berm --version
berm --help
berm cover --help
```

---

## Examples

```bash
# 1. Score a wallet's risk on devnet before designing any cover
berm scan --wallet 7xKXtg2CW87d97TXJSDpbD5jBkheTqA83TZRuJosgAsU

# 2. Price 30-day depeg protection and build an unsigned transaction
berm cover --type depeg --amount 1000 --duration 30d --wallet 7xKX...gAsU

# 3. Sign and submit a cover-intent transaction with a local keypair
berm cover --type depeg --amount 1000 --duration 30d \
  --keypair ~/.config/solana/id.json --send

# 4. Track a claim's auto-trigger state
berm claim --id clm_8f3a21

# 5. List pools and pipe machine-readable output into jq
berm pool list --json | jq '.[] | {id, tvlUsd, utilization}'

# 6. Check oracle feed health on a specific cluster
berm oracle status --cluster devnet
```

---

## Output formats

Every command supports `--json` for scripting and CI pipelines. Without it, the
CLI renders fixed-width tables in the Storm Breakwater palette (dark slate base,
glowing cyan primary, accent gold for token and reward figures).

---

## Cover types

| Type | Trigger | Oracle |
| --- | --- | --- |
| `exploit` | TVL collapse + abnormal withdrawal pattern | Pyth + on-chain TVL feed |
| `depeg` | Stable price below 0.95 or above 1.05 | Pyth + Switchboard |
| `slashing` | LST validator slashing epoch event | Solana native stake feed |
| `liquidation` | Marginfi / Kamino / Solend liquidation | Per-protocol event + Pyth |
| `oracle` | Pyth vs Switchboard divergence | Dual oracle monitor |

---

## Troubleshooting

**`error: request failed: ...` on `pool`, `oracle` or `claim`.**
The backend at `BERM_API_URL` was unreachable. Confirm the URL and your network,
or set a different endpoint: `berm pool list --api https://api.berm.sh`.

**`berm cover` prints "local estimate".**
The backend quote endpoint was unavailable, so the premium was computed locally
from the protocol rate curve. Set `BERM_API_URL` to a reachable backend for an
authoritative quote.

**`error: Invalid wallet address`.**
The value passed to `--wallet` is not a valid base58 Solana public key.

**`keypair error: Keypair file not found ...`.**
The path passed to `--keypair` does not exist or is not a Solana CLI keypair
(a JSON array of 32 or 64 bytes).

**`insufficient funds: ...` on `berm cover --send`.**
The fee payer does not hold enough lamports to cover the transaction fee. Fund
the account (on devnet, `solana airdrop 1 <ADDRESS> --url devnet`).

**`simulation error: ...` on `berm cover --send`.**
The transaction failed simulation against the cluster; the program logs are
printed below the message.

**`scan` is slow or times out.**
The default public RPC is rate-limited. Point `--rpc` at a faster endpoint you
control, or set `BERM_RPC_URL`.

**`command not found: berm` after install.**
Ensure your global npm bin directory is on `PATH` (`npm bin -g`), or re-run the
install with appropriate permissions.

---

## Development

```bash
npm install
npm run build      # bundle to dist/ with tsup
npm run typecheck  # tsc --noEmit
npm pack           # produce a publishable .tgz
```

---

## License

MIT
