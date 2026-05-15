import { create } from "zustand";
import {
  api,
  type AlertEvent,
  type CoverPosition,
  type ProtocolStats,
} from "../lib/api";
import { scoreWallet, type RiskReport } from "../lib/risk";
import { connectWallet, disconnectWallet, getStoredAddress } from "../lib/wallet";
import { subscribeWallet } from "../lib/push";
import { buildSnapshot, persistSnapshot } from "../lib/watch";

interface AsyncSlice<T> {
  data: T | null;
  loading: boolean;
  error: string | null;
}

function idleSlice<T>(): AsyncSlice<T> {
  return { data: null, loading: false, error: null };
}

interface StoreState {
  wallet: string | null;
  connecting: boolean;
  stats: AsyncSlice<ProtocolStats>;
  positions: AsyncSlice<CoverPosition[]>;
  alerts: AsyncSlice<AlertEvent[]>;
  risk: AsyncSlice<RiskReport>;

  restore: () => Promise<void>;
  connect: () => Promise<void>;
  disconnect: () => Promise<void>;
  refreshStats: () => Promise<void>;
  refreshWalletData: () => Promise<void>;
  unreadCount: () => number;
}

function errorMessage(err: unknown): string {
  return err instanceof Error ? err.message : String(err);
}

export const useStore = create<StoreState>((set, get) => ({
  wallet: null,
  connecting: false,
  stats: idleSlice(),
  positions: idleSlice(),
  alerts: idleSlice(),
  risk: idleSlice(),

  restore: async () => {
    const address = await getStoredAddress();
    if (address) {
      set({ wallet: address });
      await get().refreshWalletData();
    }
    await get().refreshStats();
  },

  connect: async () => {
    set({ connecting: true });
    try {
      const { address } = await connectWallet();
      set({ wallet: address, connecting: false });
      void subscribeWallet(address);
      await get().refreshWalletData();
    } catch (err) {
      set({ connecting: false });
      throw new Error(errorMessage(err));
    }
  },

  disconnect: async () => {
    await disconnectWallet();
    set({ wallet: null, positions: idleSlice(), alerts: idleSlice(), risk: idleSlice() });
  },

  refreshStats: async () => {
    set((s) => ({ stats: { ...s.stats, loading: true, error: null } }));
    try {
      const data = await api.stats();
      set({ stats: { data, loading: false, error: null } });
    } catch (err) {
      set((s) => ({ stats: { ...s.stats, loading: false, error: errorMessage(err) } }));
    }
  },

  refreshWalletData: async () => {
    const wallet = get().wallet;
    if (!wallet) return;

    set((s) => ({
      positions: { ...s.positions, loading: true, error: null },
      alerts: { ...s.alerts, loading: true, error: null },
      risk: { ...s.risk, loading: true, error: null },
    }));

    // Risk is computed directly from chain, so it succeeds even if the backend
    // is down. Positions and alerts come from the backend independently.
    const [riskResult, positionsResult, alertsResult] = await Promise.allSettled([
      scoreWallet(wallet),
      api.positions(wallet),
      api.alerts(wallet),
    ]);

    set({
      risk:
        riskResult.status === "fulfilled"
          ? { data: riskResult.value, loading: false, error: null }
          : { data: null, loading: false, error: errorMessage(riskResult.reason) },
      positions:
        positionsResult.status === "fulfilled"
          ? { data: positionsResult.value, loading: false, error: null }
          : { data: null, loading: false, error: errorMessage(positionsResult.reason) },
      alerts:
        alertsResult.status === "fulfilled"
          ? { data: alertsResult.value, loading: false, error: null }
          : { data: null, loading: false, error: errorMessage(alertsResult.reason) },
    });

    // Update the watch / wearable snapshot from the freshest values.
    const next = get();
    void persistSnapshot(
      buildSnapshot({
        risk: next.risk.data,
        positions: next.positions.data,
        alerts: next.alerts.data,
      }),
    );
  },

  unreadCount: () => {
    const alerts = get().alerts.data ?? [];
    return alerts.filter((a) => !a.read).length;
  },
}));
