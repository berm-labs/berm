import React, { useCallback, useState } from "react";
import { StyleSheet, Text, View } from "react-native";
import { ScreenContainer } from "../components/ScreenContainer";
import { Card, SectionTitle } from "../components/Card";
import { RiskGauge } from "../components/RiskGauge";
import { StatTile } from "../components/StatTile";
import { StatusPill } from "../components/StatusPill";
import { Button } from "../components/Button";
import { EmptyState, ErrorState, Loading } from "../components/StateView";
import { useStore } from "../state/useStore";
import { colors } from "../theme/colors";
import { typography } from "../theme/typography";
import { compactUsd, num, shortAddress } from "../lib/format";

export function HomeScreen() {
  const wallet = useStore((s) => s.wallet);
  const connecting = useStore((s) => s.connecting);
  const stats = useStore((s) => s.stats);
  const risk = useStore((s) => s.risk);
  const positions = useStore((s) => s.positions);
  const connect = useStore((s) => s.connect);
  const refreshStats = useStore((s) => s.refreshStats);
  const refreshWalletData = useStore((s) => s.refreshWalletData);

  const [refreshing, setRefreshing] = useState(false);
  const [connectError, setConnectError] = useState<string | null>(null);

  const onRefresh = useCallback(async () => {
    setRefreshing(true);
    await Promise.all([refreshStats(), refreshWalletData()]);
    setRefreshing(false);
  }, [refreshStats, refreshWalletData]);

  const onConnect = useCallback(async () => {
    setConnectError(null);
    try {
      await connect();
    } catch (err) {
      setConnectError(err instanceof Error ? err.message : String(err));
    }
  }, [connect]);

  const activeCovers = positions.data?.filter((p) => p.state === "active").length ?? 0;

  return (
    <ScreenContainer refreshing={refreshing} onRefresh={onRefresh}>
      <View style={styles.masthead}>
        <Text style={styles.brand}>BERM Alert</Text>
        <Text style={styles.tagline}>Break the wave.</Text>
      </View>

      <SectionTitle>Protocol</SectionTitle>
      {stats.loading && !stats.data ? (
        <Loading label="Loading protocol stats" />
      ) : stats.error && !stats.data ? (
        <ErrorState message={stats.error} />
      ) : stats.data ? (
        <View style={styles.grid}>
          <StatTile label="Covers active" value={num(stats.data.coversActive, 0)} />
          <StatTile label="TVL" value={compactUsd(stats.data.tvlUsd)} />
          <StatTile label="Cover types" value={num(stats.data.coverTypes, 0)} />
          <StatTile label="Triggers" value={num(stats.data.triggeredEvents, 0)} />
        </View>
      ) : null}

      <SectionTitle>Your position</SectionTitle>
      {!wallet ? (
        <Card>
          <Text style={styles.connectCopy}>
            Connect a Solana wallet to read your live position risk and receive cover alerts.
          </Text>
          <Button label={connecting ? "Connecting" : "Connect wallet"} onPress={onConnect} loading={connecting} />
          {connectError ? <Text style={styles.error}>{connectError}</Text> : null}
        </Card>
      ) : (
        <Card>
          <View style={styles.walletRow}>
            <Text style={styles.walletAddr}>{shortAddress(wallet)}</Text>
            <StatusPill
              label={activeCovers > 0 ? `${activeCovers} active covers` : "no active cover"}
              tone={activeCovers > 0 ? "amber" : "muted"}
            />
          </View>

          {risk.loading && !risk.data ? (
            <Loading label="Scanning position" />
          ) : risk.error && !risk.data ? (
            <ErrorState message={risk.error} />
          ) : risk.data ? (
            <View style={styles.gaugeWrap}>
              <RiskGauge score={risk.data.score} />
              <View style={styles.subScores}>
                <SubScore label="Depeg" value={risk.data.scores.depeg} />
                <SubScore label="Slashing" value={risk.data.scores.slashing} />
                <SubScore label="Concentration" value={risk.data.scores.concentration} />
              </View>
            </View>
          ) : (
            <EmptyState title="No risk data" hint="Pull to refresh to rescan your wallet." />
          )}
        </Card>
      )}
    </ScreenContainer>
  );
}

function SubScore({ label, value }: { label: string; value: number }) {
  return (
    <View style={styles.subScore}>
      <Text style={styles.subScoreLabel}>{label}</Text>
      <Text style={styles.subScoreValue}>{Math.round(value)}</Text>
    </View>
  );
}

const styles = StyleSheet.create({
  masthead: {
    marginTop: 4,
    gap: 2,
  },
  brand: {
    ...typography.display,
    color: colors.whiteGlow,
    fontSize: 28,
  },
  tagline: {
    ...typography.mono,
    color: colors.glowCyan,
    fontSize: 13,
    letterSpacing: 1,
  },
  grid: {
    flexDirection: "row",
    flexWrap: "wrap",
    gap: 12,
  },
  connectCopy: {
    ...typography.body,
    color: colors.textMuted,
    fontSize: 14,
    marginBottom: 4,
  },
  walletRow: {
    flexDirection: "row",
    justifyContent: "space-between",
    alignItems: "center",
  },
  walletAddr: {
    ...typography.mono,
    color: colors.fogWhite,
    fontSize: 15,
  },
  gaugeWrap: {
    alignItems: "center",
    gap: 16,
    marginTop: 8,
  },
  subScores: {
    flexDirection: "row",
    justifyContent: "space-around",
    width: "100%",
  },
  subScore: {
    alignItems: "center",
    gap: 2,
  },
  subScoreLabel: {
    ...typography.body,
    color: colors.textMuted,
    fontSize: 12,
  },
  subScoreValue: {
    ...typography.mono,
    color: colors.textPrimary,
    fontSize: 18,
  },
  error: {
    ...typography.body,
    color: colors.danger,
    fontSize: 13,
    marginTop: 8,
  },
});
