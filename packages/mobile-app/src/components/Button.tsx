import React from "react";
import { ActivityIndicator, Pressable, StyleSheet, Text } from "react-native";
import * as Haptics from "expo-haptics";
import { colors } from "../theme/colors";
import { typography } from "../theme/typography";

interface Props {
  label: string;
  onPress: () => void;
  loading?: boolean;
  variant?: "primary" | "ghost";
}

export function Button({ label, onPress, loading, variant = "primary" }: Props) {
  const primary = variant === "primary";
  return (
    <Pressable
      onPress={() => {
        void Haptics.impactAsync(Haptics.ImpactFeedbackStyle.Light);
        onPress();
      }}
      disabled={loading}
      style={({ pressed }) => [
        styles.base,
        primary ? styles.primary : styles.ghost,
        pressed && styles.pressed,
      ]}
    >
      {loading ? (
        <ActivityIndicator color={primary ? colors.stormNavy : colors.glowCyan} />
      ) : (
        <Text style={[styles.label, primary ? styles.primaryLabel : styles.ghostLabel]}>{label}</Text>
      )}
    </Pressable>
  );
}

const styles = StyleSheet.create({
  base: {
    height: 50,
    borderRadius: 12,
    alignItems: "center",
    justifyContent: "center",
    paddingHorizontal: 20,
  },
  primary: {
    backgroundColor: colors.glowCyan,
  },
  ghost: {
    borderWidth: 1,
    borderColor: colors.glowCyan,
  },
  pressed: {
    opacity: 0.8,
  },
  label: {
    ...typography.title,
    fontSize: 15,
    letterSpacing: 0.5,
  },
  primaryLabel: {
    color: colors.stormNavy,
  },
  ghostLabel: {
    color: colors.glowCyan,
  },
});
