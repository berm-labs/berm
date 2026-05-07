import React from "react";
import { StyleSheet, Text, View } from "react-native";
import { Card } from "./Card";
import { StatusPill } from "./StatusPill";
import { colors } from "../theme/colors";
import { typography } from "../theme/typography";
import { usd } from "../lib/format";
import type { CoverPosition } from "../lib/api";

const STATE_TONE = {
  active: "ok",
  triggered: "amber",
  expired: "muted",
} as const;

export function CoverCard({ position }: { position: CoverPosition }) {
  return (
    <Card>
      <View style={styles.row}>
        <Text style={styles.label}>{position.label}</Text>
        <StatusPill label={position.state} tone={STATE_TONE[position.state]} />
      </View>
      <View style={styles.metrics}>
        <Metric label="Cover" value={usd(position.amountUsd)} />
        <Metric label="Premium" value={usd(position.premiumUsd)} accent />
        <Metric label="Risk" value={`${position.riskScore}`} />
      </View>
      <Text style={styles.expiry}>Expires {position.expiresAt.slice(0, 10)}</Text>
    </Card>
  );
}

function Metric({ label, value, accent }: { label: string; value: string; accent?: boolean }) {
  return (
    <View style={styles.metric}>
      <Text style={styles.metricLabel}>{label}</Text>
      <Text style={[styles.metricValue, accent && { color: colors.accentGold }]}>{value}</Text>
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
    gap: 20,
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
  expiry: {
    ...typography.body,
    color: colors.textMuted,
    fontSize: 12,
  },
});
