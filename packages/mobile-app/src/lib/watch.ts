import AsyncStorage from "@react-native-async-storage/async-storage";
import { riskBand } from "../theme/colors";
import type { RiskReport } from "./risk";
import type { CoverPosition, AlertEvent } from "./api";

// Canonical snapshot shared with the Apple Watch complication and Wear OS tile.
// The shape matches CoverSnapshot in watch/apple-watch and watch/wear-os.
export interface CoverSnapshot {
  riskScore: number;
  band: string;
  activeCovers: number;
  unreadAlerts: number;
  updatedAt: string;
}

export const WATCH_SNAPSHOT_KEY = "berm.coverSnapshot";

export function buildSnapshot(input: {
  risk: RiskReport | null;
  positions: CoverPosition[] | null;
  alerts: AlertEvent[] | null;
}): CoverSnapshot {
  const riskScore = input.risk?.score ?? 0;
  return {
    riskScore,
    band: riskBand(riskScore),
    activeCovers: input.positions?.filter((p) => p.state === "active").length ?? 0,
    unreadAlerts: input.alerts?.filter((a) => !a.read).length ?? 0,
    updatedAt: new Date().toISOString(),
  };
}

// Persists the snapshot. AsyncStorage is the source of truth read by the native
// widget bridge (config plugin) which mirrors it into the iOS App Group and the
// Wear Data Layer. See watch/README.md for the bridge contract.
export async function persistSnapshot(snapshot: CoverSnapshot): Promise<void> {
  await AsyncStorage.setItem(WATCH_SNAPSHOT_KEY, JSON.stringify(snapshot));
}
