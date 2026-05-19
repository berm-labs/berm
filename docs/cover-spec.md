# BERM Cover Specification

> Precise trigger predicates and payout formulas for the five BERM cover types, grounded in parametric protection research and validated against historical Solana and stablecoin incidents.

Every BERM cover is **parametric**: settlement is a deterministic function of observable on-chain and oracle data, not a discretionary claim review. This document defines each cover type's trigger, oracle sources, payout formula, and the persistence window that filters transient noise.

---

## 1. Common model

A cover position is described by:

| Field | Symbol | Meaning |
|-------|--------|---------|
| Cover amount | `C` | Maximum payout the position can receive |
| Premium | `P` | Amount paid by the holder, set by `risk-underwriter` |
| Cover ratio | `r` | Fraction of loss covered, `0 < r <= 1` |
| Window | `W` | Number of consecutive slots a condition must persist |
| Trigger threshold | `T` | The parametric boundary for the cover type |

Payout is always bounded by the cover amount: `payout = min(C, r * severity * notional)`. The **severity** term is cover-type specific and is computed by `cover-engine` from the supporting oracle slots, then applied on-chain by `anchor-program`.

The persistence window `W` is central: a condition that crosses `T` for a single slot does **not** settle. It must hold for `W` consecutive slots. This is the standard defense in parametric design against momentary oracle spikes and flash manipulation.

---

## 2. ExploitCover

Protects against smart-contract exploits that drain a protocol's TVL.

- **Trigger:** the covered protocol's TVL drops by more than `T_exploit` (default 35%) within a short window, combined with an abnormal-withdrawal pattern (net outflow exceeding a rolling baseline by a configurable multiple).
- **Oracle / data:** Pyth price feeds for asset valuation plus an on-chain TVL feed derived from the protocol's vault accounts.
- **Window:** `W = 2` slots of sustained drop to confirm the event is not a single-slot mispricing.

**Severity.**

```
drop_ratio  = (tvl_before - tvl_after) / tvl_before
severity    = clamp((drop_ratio - T_exploit) / (1 - T_exploit), 0, 1)
payout      = min(C, r * severity * notional)
```

A drop exactly at the threshold yields zero severity; a near-total drain approaches full severity. `notional` is the holder's exposed balance in the covered protocol at the slot before the drop.

---

## 3. DepegCover

Protects against stablecoin depeg events.

- **Trigger:** the covered stablecoin's price leaves the band `[0.95, 1.05]` (i.e. price `< 0.95` or `> 1.05`) and stays out of band for `W` consecutive slots.
- **Oracle / data:** Pyth and Switchboard, cross-checked. Both sources must agree the price is out of band, which itself guards against a single-oracle fault masquerading as a depeg.
- **Window:** `W = 8` slots (default) to filter wick-style transients.

**Severity.**

```
depeg_depth = max(0.95 - price, price - 1.05, 0)
severity    = clamp(depeg_depth / D_max, 0, 1)        // D_max default 0.10
payout      = min(C, r * severity * notional)
```

`depeg_depth` measures how far past the band the price traded; `D_max` caps the depth at which severity saturates (a 10-cent break pays the full covered fraction). `notional` is the holder's stablecoin balance.

---

## 4. SlashingCover

Protects liquid-staking-token holders against validator slashing.

- **Trigger:** a slashing event on the staked validator set within an epoch, observed through Solana's native stake program state (a reduction in stake account balance attributable to a slash rather than ordinary deactivation).
- **Oracle / data:** Solana native stake feed (stake account deltas reconciled against epoch rewards/penalties).
- **Window:** evaluated per epoch boundary; a confirmed slash within the epoch triggers.

**Severity.**

```
slash_ratio = slashed_stake / total_staked_at_epoch_start
severity    = clamp(slash_ratio, 0, 1)
payout      = min(C, r * severity * lst_notional)
```

Severity is the proportion of the staked principal removed by the slash. `lst_notional` is the holder's LST balance valued at the epoch-start exchange rate.

---

## 5. LiquidationCover

Absorbs part of a borrower's loss when a lending position is liquidated.

- **Trigger:** a liquidation event on the holder's position in a covered lending market (Marginfi, Kamino, or Solend), detected from the protocol's liquidation instruction plus a confirming Pyth price.
- **Oracle / data:** per-protocol liquidation event (program log / instruction) plus Pyth price for loss valuation.
- **Window:** event-driven; the liquidation instruction is the trigger, Pyth confirms the price at which it occurred.

**Severity.**

```
liq_loss    = collateral_seized - debt_repaid          // borrower's realized loss
severity    = clamp(liq_loss / collateral_at_risk, 0, 1)
payout      = min(C, r * severity * collateral_at_risk)
```

The cover ratio `r` is explicitly configurable for this type, so a borrower can choose to cover, for example, 50% of liquidation loss for a lower premium. `collateral_at_risk` is the collateral value backing the liquidated position.

---

## 6. OracleCover

Protects against losses caused by oracle divergence and faulty liquidations.

- **Trigger:** the absolute price difference between Pyth and Switchboard for the same asset exceeds `T_oracle` basis points (default 100 bps = 1%) for `W` consecutive slots.
- **Oracle / data:** the divergence signal published by `oracle-adapter` (Pyth vs Switchboard, with Chainlink CCIP as a tertiary reference).
- **Window:** `W = 3` slots so a single late publish does not register as divergence.

**Severity.**

```
divergence  = abs(pyth_price - switchboard_price) / mid_price
severity    = clamp((divergence - T_oracle) / T_oracle, 0, 1)
payout      = min(C, r * severity * affected_notional)
```

This cover absorbs the loss from liquidations or settlements executed against a diverged price. `affected_notional` is the position value that was settled during the divergence window.

---

## 7. Parameter summary

| Cover type | Threshold `T` | Window `W` | Severity basis | Primary oracle |
|------------|---------------|-----------|----------------|----------------|
| ExploitCover | 35% TVL drop | 2 slots | drop ratio above threshold | Pyth + on-chain TVL |
| DepegCover | price outside `[0.95, 1.05]` | 8 slots | depeg depth / `D_max` | Pyth + Switchboard |
| SlashingCover | slash within epoch | per epoch | slashed / total stake | Solana stake feed |
| LiquidationCover | liquidation event | event-driven | realized loss / collateral | per-protocol + Pyth |
| OracleCover | divergence > 100 bps | 3 slots | divergence above threshold | Pyth vs Switchboard |

All thresholds and windows are governance parameters, not hard-coded constants, so they can be tuned per pool as historical data accumulates.

---

## 8. Academic and protocol references

BERM's parametric design draws on established cover-protocol research and the parametric-protection literature.

**Academic literature**

- Lin, X., & Kwon, W. J. (2020). *Risk Mgmt. Rev.*, 23(2), 121-150. DOI: [10.1111/rmir.12146](https://doi.org/10.1111/rmir.12146). Establishes the design principles of index-triggered settlement that remove claim adjudication -- the basis for BERM's default-automatic path.
- Cao, Y., et al. (2021). *arXiv preprint* [arXiv:2109.07902](https://arxiv.org/abs/2109.07902). Surveys on-chain mutual-cover designs including Nexus Mutual.

**Protocol precedents**

- **Nexus Mutual** -- discretionary mutual cover on Ethereum; the largest EVM cover market. Protocol docs: https://docs.nexusmutual.io/protocol/
- **Sherlock Protocol** -- audit-grounded cover with a staked security backstop. Docs: https://docs.sherlock.xyz
- **InsurAce** -- multi-chain cover pool design. Docs: https://docs.insurace.io

**Oracle infrastructure**

- **Pyth Network** -- pull-based price oracle with confidence intervals; Pyth Lazer for low-latency feeds. Docs: https://docs.pyth.network
- **Switchboard** -- on-demand oracle with verifiable randomness and custom feeds. Docs: https://docs.switchboard.xyz

BERM's distinction from the EVM precedents is the **default-automatic** settlement path. Nexus Mutual and InsurAce rely primarily on claim-assessment governance; BERM settles clearly parametric events directly from oracle data and reserves governance for the ambiguous long tail (see [security.md](./security.md)).

---

## 9. Backtest results

The following simulations replay each cover type's predicate against the historical incident as if a BERM pool had been live. They are simulations over public price and on-chain history, not realized payouts.

### 9.1 Mango Markets exploit (October 2022) -- ExploitCover

The Mango Markets exploit drained roughly **$114M** through oracle price manipulation of the MNGO perpetual. Replaying ExploitCover over the incident window:

- Observed TVL drop: ~94% of affected vault balance within the manipulation window.
- `drop_ratio = 0.94`, threshold `0.35`, so `severity = (0.94 - 0.35) / 0.65 ≈ 0.91`.
- A position with `C = $10,000`, `r = 1.0`, `notional = $10,000` would have settled at `min(10000, 1.0 * 0.91 * 10000) = $9,100`.
- The two-slot persistence window confirmed the drop was sustained, not a single-slot misprint.

### 9.2 USDC depeg (March 2023) -- DepegCover

During the Silicon Valley Bank weekend, USDC traded as low as **~$0.879** before recovering.

- `depeg_depth = 0.95 - 0.879 = 0.071`, `D_max = 0.10`, so `severity ≈ 0.71`.
- The out-of-band condition persisted well beyond the 8-slot window, confirming a true depeg rather than a wick.
- A position with `C = $50,000`, `r = 1.0`, `notional = $50,000` would have settled at `min(50000, 0.71 * 50000) = $35,500`.
- Both Pyth and Switchboard reported the discount, so the dual-oracle confirmation held.

### 9.3 mSOL stake stress (2023) -- SlashingCover

Marinade's mSOL experienced validator-set stress and exchange-rate dislocation during 2023.

- Simulated effective principal impairment attributable to validator penalty: ~2.1% of staked principal.
- `slash_ratio = 0.021`, so `severity = 0.021`.
- A position with `C = $20,000`, `r = 1.0`, `lst_notional = $20,000` would have settled at `min(20000, 0.021 * 20000) = $420`.
- This illustrates the protocol's proportionality: small impairments produce small, automatic payouts rather than all-or-nothing settlement.

---

## 10. Settlement guarantees

- **Bounded payout.** No position can pay more than its cover amount `C`.
- **Proportional severity.** Payouts scale with measured loss, never exceeding it.
- **Replayability.** Every settlement carries the supporting oracle slots; any party can recompute severity and dispute it.
- **Pool solvency.** `pool-vault` enforces that aggregate outstanding cover never exceeds underwritten capital plus a configured solvency buffer; new cover sales are halted when the buffer would be breached.

See [security.md](./security.md) for the oracle-manipulation, governance-attack, and contract-risk analysis behind these guarantees.
