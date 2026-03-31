# @berm/sdk

TypeScript SDK for **BERM**, Solana's first parametric DeFi cover protocol. Read cover pools, quote and buy cover positions, evaluate parametric triggers, score protocol risk, read dual-oracle prices, and backtest against historical incidents.

Site: [berm.sh](https://berm.sh) &middot; GitHub: [berm-labs/berm](https://github.com/berm-labs/berm) &middot; X: [@berm_sh](https://x.com/berm_sh)

> The program is currently deployed to Solana devnet; the SDK defaults to the public devnet RPC. Pass `endpoint`/`programId` to target another cluster.

BERM provides oracle-triggered automatic settlement across five cover types: ExploitCover, DepegCover, SlashingCover, LiquidationCover, and OracleCover. See [`docs/cover-spec.md`](../../docs/cover-spec.md) for trigger predicates and payout formulas, and [`docs/architecture.md`](../../docs/architecture.md) for the system design.

## Install

```bash
npm install @berm/sdk
```

Peer stack: `@solana/web3.js`, `@coral-xyz/anchor`, `@solana-program/token-2022`.

## Quick start

```ts
import { BermClient } from "@berm/sdk";

const berm = new BermClient({
  endpoint: "https://api.devnet.solana.com",
});

const pools = await berm.pools.fetchAll();
for (const pool of pools) {
  console.log(pool.coverType, pool.totalCapital.toString());
}
```

## API surface

| Class | Responsibility |
|-------|----------------|
| `BermClient` | Entry point; holds the RPC connection and all sub-clients. |
| `CoverPool` | Read and create cover pools, underwrite liquidity. |
| `CoverPosition` | Buy cover, query positions, burn positions. |
| `ClaimResolver` | Read claim records, evaluate triggers, propose settlement. |
| `RiskScorer` | Score protocol risk and quote premiums. |
| `OracleAdapter` | Read Pyth prices and compute dual-oracle divergence. |

## Examples

### 1. Buy cover

```ts
import { BermClient, CoverType } from "@berm/sdk";
import { PublicKey, Transaction } from "@solana/web3.js";
import BN from "bn.js";

const berm = new BermClient();
const usdc = new PublicKey("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");

// Pool for DepegCover on USDC.
const pool = await berm.pools.fetch(CoverType.Depeg, usdc);
if (!pool) throw new Error("pool not found");

const buyer = wallet.publicKey;
const ix = berm.positions.buyCoverIx({
  buyer,
  pool: pool.address,
  index: 0,
  buyerTokenAccount: buyerUsdcAccount,
  coverAmount: new BN(50_000_000_000), // 50,000 USDC (6 decimals)
  coverRatioBps: 10_000, // 100%
  durationSlots: new BN(30 * 216_000), // ~30 days
  premium: new BN(1_250_000_000),
});

const tx = new Transaction().add(ix);
await wallet.sendTransaction(tx, berm.connection);
```

### 2. Query positions for a wallet

```ts
import { BermClient, COVER_TYPE_LABEL } from "@berm/sdk";
import { PublicKey } from "@solana/web3.js";

const berm = new BermClient();
const owner = new PublicKey("9xQeWvG816bUx9EPjHmaT23yvVM2ZWbrrpZb9PusVFin");

const positions = await berm.positions.fetchByOwner(owner);
const slot = await berm.currentSlot();

for (const p of positions) {
  console.log(
    COVER_TYPE_LABEL[p.coverType],
    "cover:", p.coverAmount.toString(),
    "active:", CoverPosition.isActive(p, new BN(slot))
  );
}
```

### 3. Evaluate and trigger a claim

```ts
import { BermClient, CoverType } from "@berm/sdk";
import BN from "bn.js";

const berm = new BermClient();
const position = await berm.positions.fetch(poolPubkey, owner, 0);
if (!position) throw new Error("no position");

// Mirror the on-chain predicate: a USDC depeg to $0.93.
const evaluation = berm.claims.evaluate(position, {
  coverType: CoverType.Depeg,
  price: 0.93,
  notional: new BN(50_000_000_000),
});

if (evaluation.triggered) {
  const ix = berm.claims.triggerClaimIx({
    cranker: wallet.publicKey,
    pool: poolPubkey,
    position: position.address,
    severityBps: Math.round(evaluation.severity * 10_000),
  });
  // submit ix...
}
```

### 4. Score protocol risk and quote a premium

```ts
import { BermClient, CoverType } from "@berm/sdk";
import BN from "bn.js";

const berm = new BermClient();

const risk = berm.risk.score({
  protocol: "Kamino",
  tvlUsd: 2_000_000_000,
  auditCount: 3,
  codeComplexity: 60,
  reputation: 80,
});

const quote = berm.risk.quote(
  CoverType.Liquidation,
  new BN(10_000_000_000), // 10,000 cover
  30, // days
  risk
);

console.log("risk score:", risk.score, "premium:", quote.premium.toString());
```

### 5. Read dual-oracle prices and divergence

```ts
import { OracleAdapter } from "@berm/sdk";

const oracle = new OracleAdapter();

// USDC/USD Pyth feed id.
const pyth = await oracle.getPythPrice(
  "eaa020c61cc479712813461ce153894a96a6c00b21ed0cfc2798d1f9a9e9c94a"
);

const aggregated = oracle.aggregate(pyth, {
  price: 0.998,
  confidence: 0.001,
  publishSlot: pyth.publishSlot,
  source: "switchboard",
});

console.log("mid:", aggregated.mid, "diverged:", aggregated.diverged);
```

### 6. Backtest against a historical incident

```ts
import { backtest, SCENARIOS } from "@berm/sdk";
import BN from "bn.js";

// Replay the March 2023 USDC depeg against a 50,000-USDC position.
const result = backtest("usdc2023", new BN(50_000), 10_000, new BN(50_000));
console.log(result.scenario, "payout:", result.payout.toString());
// "USDC depeg (Mar 2023) payout: 35500"

console.log(Object.keys(SCENARIOS)); // mango2022, usdc2023, msol2023
```

## Cover types

| Type | Enum | Trigger |
|------|------|---------|
| ExploitCover | `CoverType.Exploit` | TVL drop past threshold over a slot window |
| DepegCover | `CoverType.Depeg` | Stablecoin price outside `[0.95, 1.05]` for N slots |
| SlashingCover | `CoverType.Slashing` | Validator slashing within an epoch |
| LiquidationCover | `CoverType.Liquidation` | Liquidation on a covered lending position |
| OracleCover | `CoverType.Oracle` | Pyth vs Switchboard divergence past threshold |

## PDA helpers

```ts
import { coverPoolPda, coverPositionPda, claimPda, CoverType } from "@berm/sdk";
import { PublicKey } from "@solana/web3.js";

const usdc = new PublicKey("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");
const [pool] = coverPoolPda(CoverType.Depeg, usdc);
const [position] = coverPositionPda(pool, owner, 0);
const [claim] = claimPda(position);
```

## Severity math

The severity functions are exported directly so clients can compute payouts without an RPC round trip. They mirror the on-chain logic exactly.

```ts
import { depegSeverity, payout } from "@berm/sdk";
import BN from "bn.js";

const severity = depegSeverity(0.93); // depth past the band, normalized
const settled = payout(new BN(50_000), 10_000, severity, new BN(50_000));
```

## Build

```bash
npm run build   # tsc --build -> dist/
```

## Test

The severity math, PDA derivation, risk scoring, instruction codecs, and oracle
aggregation are covered by a Vitest suite (55 assertions). The backtest cases
assert the exact payouts published in `docs/cover-spec.md`.

```bash
npm test
```

## License

MIT
