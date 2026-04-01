import BN from "bn.js";
import { CoverType } from "./constants";
import { BacktestResult } from "./types";
import {
  depegSeverity,
  exploitSeverity,
  slashingSeverity,
  payout,
} from "./severity";

/** A historical incident replayable against a cover position. */
export interface Scenario {
  name: string;
  coverType: CoverType;
  /** Severity computed from the historical incident data. */
  severity: number;
}

/**
 * Canonical historical scenarios, with severities derived from the public
 * record per docs/cover-spec.md section 9.
 */
export const SCENARIOS: Record<string, Scenario> = {
  mango2022: {
    name: "Mango Markets exploit (Oct 2022)",
    coverType: CoverType.Exploit,
    severity: exploitSeverity(0.94),
  },
  usdc2023: {
    name: "USDC depeg (Mar 2023)",
    coverType: CoverType.Depeg,
    severity: depegSeverity(0.879),
  },
  msol2023: {
    name: "mSOL stake stress (2023)",
    coverType: CoverType.Slashing,
    severity: slashingSeverity(2.1, 100),
  },
};

/**
 * Replay a scenario against a hypothetical position and return the payout
 * the cover would have produced.
 */
export function backtest(
  scenarioKey: keyof typeof SCENARIOS | Scenario,
  coverAmount: BN,
  coverRatioBps: number,
  notional: BN
): BacktestResult {
  const scenario =
    typeof scenarioKey === "string" ? SCENARIOS[scenarioKey] : scenarioKey;
  if (!scenario) {
    throw new Error(`Unknown scenario: ${String(scenarioKey)}`);
  }
  const settled = payout(coverAmount, coverRatioBps, scenario.severity, notional);
  return {
    coverType: scenario.coverType,
    scenario: scenario.name,
    severity: scenario.severity,
    payout: settled,
    triggered: scenario.severity > 0,
  };
}
