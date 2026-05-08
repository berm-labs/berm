import type { CoverType } from "./constants";

// Historical DeFi loss events used to backtest cover payouts. Each entry cites a
// public, documented event and the observed loss magnitude. Payouts here are
// deterministic simulations over those magnitudes, clearly labelled as such in
// the UI -- they are not live claims.
export interface BacktestScenario {
  id: string;
  coverType: CoverType;
  title: string;
  date: string;
  // Observed loss as a fraction of the exposed notional (0..1).
  lossFraction: number;
  summary: string;
  reference: string;
}

export const BACKTEST_SCENARIOS: BacktestScenario[] = [
  {
    id: "mango-2022",
    coverType: "exploit",
    title: "Mango Markets exploit",
    date: "Oct 2022",
    lossFraction: 1.0,
    summary:
      "An oracle-price manipulation drained roughly $114M, wiping pooled deposits. ExploitCover triggers on TVL collapse and abnormal withdrawals.",
    reference: "Mango Markets, October 2022 (~$114M)",
  },
  {
    id: "usdc-2023",
    coverType: "depeg",
    title: "USDC depeg",
    date: "Mar 2023",
    lossFraction: 0.1226,
    summary:
      "During the SVB crisis USDC fell to about $0.8774. DepegCover triggers below 0.95 and pays the depeg depth on the covered notional.",
    reference: "USDC, 11 March 2023 (low ~$0.8774)",
  },
  {
    id: "lst-2023",
    coverType: "slashing",
    title: "mSOL liquidity depeg",
    date: "Dec 2023",
    lossFraction: 0.07,
    summary:
      "A large forced swap pushed mSOL roughly 7% below its fair value, hitting LST holders. SlashingCover absorbs LST value loss for stakers.",
    reference: "mSOL, December 2023 (~7% transient depeg)",
  },
];

export interface BacktestResult extends BacktestScenario {
  coverAmountUsd: number;
  payoutUsd: number;
}

// Computes the simulated payout for a hypothetical cover notional.
export function runBacktest(coverAmountUsd: number): BacktestResult[] {
  return BACKTEST_SCENARIOS.map((s) => ({
    ...s,
    coverAmountUsd,
    payoutUsd: Math.round(coverAmountUsd * s.lossFraction * 100) / 100,
  }));
}
