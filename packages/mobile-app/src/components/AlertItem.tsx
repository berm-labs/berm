import React from "react";
import { StyleSheet, Text, View } from "react-native";
import { colors } from "../theme/colors";
import { typography } from "../theme/typography";
import { timeAgo } from "../lib/format";
import type { AlertEvent } from "../lib/api";

const KIND_COLOR: Record<AlertEvent["kind"], string> = {
  depeg: colors.safetyAmber,
  liquidation: colors.warn,
  claim: colors.glowCyan,
  risk: colors.accentGold,
};

export function AlertItem({ alert }: { alert: AlertEvent }) {
  const color = KIND_COLOR[alert.kind];
  return (
    <View style={styles.row}>
      <View style={[styles.rail, { backgroundColor: color }]} />
      <View style={styles.body}>
        <View style={styles.header}>
          <Text style={[styles.title, !alert.read && styles.unread]}>{alert.title}</Text>
          <Text style={styles.time}>{timeAgo(alert.createdAt)}</Text>
        </View>
        <Text style={styles.text}>{alert.body}</Text>
      </View>
    </View>
  );
}

const styles = StyleSheet.create({
  row: {
    flexDirection: "row",
    backgroundColor: colors.surface,
    borderRadius: 12,
    borderWidth: 1,
    borderColor: colors.border,
    overflow: "hidden",
  },
  rail: {
    width: 4,
  },
  body: {
    flex: 1,
    padding: 12,
    gap: 4,
  },
  header: {
    flexDirection: "row",
    justifyContent: "space-between",
    alignItems: "center",
  },
  title: {
    ...typography.title,
    color: colors.textPrimary,
    fontSize: 14,
  },
  unread: {
    color: colors.whiteGlow,
  },
  time: {
    ...typography.mono,
    color: colors.textMuted,
    fontSize: 11,
  },
  text: {
    ...typography.body,
    color: colors.textMuted,
    fontSize: 13,
  },
});
