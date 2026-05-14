import React, { useCallback, useEffect, useState } from "react";
import { StyleSheet, Text, View } from "react-native";
import { ScreenContainer } from "../components/ScreenContainer";
import { Card, SectionTitle } from "../components/Card";
import { StatusPill } from "../components/StatusPill";
import { ErrorState, Loading } from "../components/StateView";
import { api, type PoolSummary } from "../lib/api";
import { colors } from "../theme/colors";
import { typography } from "../theme/typography";
import { compactUsd, num, pct } from "../lib/format";

export function PoolsScreen() {
  const [pools, setPools] = useState<PoolSummary[] | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);
  const [refreshing, setRefreshing] = useState(false);

  const load = useCallback(async () => {
    setError(null);
    try {
      const data = await api.pools();
      setPools(data);
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    void load();
  }, [load]);

  const onRefresh = useCallback(async () => {
    setRefreshing(true);
    await load();
    setRefreshing(false);
  }, [load]);

  return (
    <ScreenContainer refreshing={refreshing} onRefresh={onRefresh}>
      <SectionTitle>Cover pools</SectionTitle>
      {loading && !pools ? (
        <Loading label="Loading cover pools" />
      ) : error && !pools ? (
        <ErrorState message={error} />
      ) : (
        pools?.map((p) => (
          <Card key={p.id}>
            <View style={styles.row}>
              <Text style={styles.label}>{p.label}</Text>
              <StatusPill label={`${pct(p.utilization)} used`} tone={p.utilization > 0.8 ? "warn" : "ok"} />
            </View>
            <View style={styles.metrics}>
              <Metric label="TVL" value={compactUsd(p.tvlUsd)} />
              <Metric label="APR" value={pct(p.premiumApr)} />
              <Metric label="Covers" value={num(p.activeCovers, 0)} />
              <Metric label="Triggers" value={num(p.triggeredEvents, 0)} />
            </View>
          </Card>
        ))
      )}
    </ScreenContainer>
  );
}

function Metric({ label, value }: { label: string; value: string }) {
  return (
    <View style={styles.metric}>
      <Text style={styles.metricLabel}>{label}</Text>
      <Text style={styles.metricValue}>{value}</Text>
    </View>
  );
}

const styles = StyleSheet.create({
  row: {
    flexDirection: "row",
    justifyContent: "space-between",
    alignItems: "center",
  },
  label: {
    ...typography.title,
    color: colors.fogWhite,
    fontSize: 16,
  },
  metrics: {
    flexDirection: "row",
    flexWrap: "wrap",
    gap: 18,
    marginTop: 4,
  },
  metric: {
    gap: 2,
  },
  metricLabel: {
    ...typography.body,
    color: colors.textMuted,
    fontSize: 11,
  },
  metricValue: {
    ...typography.mono,
    color: colors.textPrimary,
    fontSize: 15,
  },
});
