import React from "react";
import { StyleSheet, Text, View, type ViewStyle } from "react-native";
import { colors } from "../theme/colors";
import { spacing, typography } from "../theme/typography";

interface CardProps {
  children: React.ReactNode;
  style?: ViewStyle;
}

export function Card({ children, style }: CardProps) {
  return <View style={[styles.card, style]}>{children}</View>;
}

export function SectionTitle({ children }: { children: React.ReactNode }) {
  return <Text style={styles.section}>{children}</Text>;
}

const styles = StyleSheet.create({
  card: {
    backgroundColor: colors.surface,
    borderRadius: 14,
    borderWidth: 1,
    borderColor: colors.border,
    padding: spacing.md,
    gap: spacing.sm,
  },
  section: {
    ...typography.title,
    color: colors.fogWhite,
    fontSize: 13,
    letterSpacing: 1.5,
    textTransform: "uppercase",
    marginTop: spacing.sm,
  },
});
