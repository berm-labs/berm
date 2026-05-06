import React from "react";
import { StyleSheet, Text, View } from "react-native";
import { colors } from "../theme/colors";
import { typography } from "../theme/typography";

interface Props {
  label: string;
  value: string;
}

export function StatTile({ label, value }: Props) {
  return (
    <View style={styles.tile}>
      <Text style={styles.value}>{value}</Text>
      <Text style={styles.label}>{label}</Text>
    </View>
  );
}

const styles = StyleSheet.create({
  tile: {
    flex: 1,
    minWidth: "45%",
    backgroundColor: colors.surfaceRaised,
    borderRadius: 12,
    borderWidth: 1,
    borderColor: colors.border,
    padding: 14,
    gap: 4,
  },
  value: {
    ...typography.display,
    color: colors.glowCyan,
    fontSize: 22,
  },
  label: {
    ...typography.body,
    color: colors.textMuted,
    fontSize: 12,
  },
});
