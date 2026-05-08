import React from "react";
import { StyleSheet, Text, View } from "react-native";
import { colors } from "../theme/colors";
import { typography } from "../theme/typography";

interface Props {
  label: string;
  tone?: "ok" | "warn" | "danger" | "amber" | "muted";
}

const TONES: Record<NonNullable<Props["tone"]>, string> = {
  ok: colors.glowCyan,
  warn: colors.warn,
  danger: colors.danger,
  amber: colors.safetyAmber,
  muted: colors.textMuted,
};

export function StatusPill({ label, tone = "ok" }: Props) {
  const color = TONES[tone];
  return (
    <View style={[styles.pill, { borderColor: color }]}>
      <View style={[styles.dot, { backgroundColor: color }]} />
      <Text style={[styles.text, { color }]}>{label}</Text>
    </View>
  );
}

const styles = StyleSheet.create({
  pill: {
    flexDirection: "row",
    alignItems: "center",
    alignSelf: "flex-start",
    borderWidth: 1,
    borderRadius: 999,
    paddingVertical: 3,
    paddingHorizontal: 10,
    gap: 6,
  },
  dot: {
    width: 7,
    height: 7,
    borderRadius: 4,
  },
  text: {
    ...typography.mono,
    fontSize: 11,
    letterSpacing: 0.5,
  },
});
