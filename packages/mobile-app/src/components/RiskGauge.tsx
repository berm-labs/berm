import React from "react";
import { StyleSheet, Text, View } from "react-native";
import Svg, { Circle } from "react-native-svg";
import { colors, riskBand, riskColor } from "../theme/colors";
import { typography } from "../theme/typography";

interface Props {
  score: number;
  size?: number;
}

// Circular gauge in the breakwater tone. The arc fills clockwise with the risk
// colour ramp; the centre shows the score and band label.
export function RiskGauge({ score, size = 160 }: Props) {
  const stroke = 12;
  const radius = (size - stroke) / 2;
  const circumference = 2 * Math.PI * radius;
  const clamped = Math.max(0, Math.min(100, score));
  const dash = (clamped / 100) * circumference;
  const color = riskColor(clamped);

  return (
    <View style={[styles.wrap, { width: size, height: size }]}>
      <Svg width={size} height={size}>
        <Circle
          cx={size / 2}
          cy={size / 2}
          r={radius}
          stroke={colors.border}
          strokeWidth={stroke}
          fill="none"
        />
        <Circle
          cx={size / 2}
          cy={size / 2}
          r={radius}
          stroke={color}
          strokeWidth={stroke}
          fill="none"
          strokeLinecap="round"
          strokeDasharray={`${dash} ${circumference}`}
          transform={`rotate(-90 ${size / 2} ${size / 2})`}
        />
      </Svg>
      <View style={styles.center}>
        <Text style={[styles.score, { color }]}>{Math.round(clamped)}</Text>
        <Text style={[styles.band, { color }]}>{riskBand(clamped)}</Text>
        <Text style={styles.unit}>risk / 100</Text>
      </View>
    </View>
  );
}

const styles = StyleSheet.create({
  wrap: {
    alignItems: "center",
    justifyContent: "center",
  },
  center: {
    position: "absolute",
    alignItems: "center",
  },
  score: {
    ...typography.display,
    fontSize: 44,
  },
  band: {
    ...typography.mono,
    fontSize: 13,
    letterSpacing: 2,
  },
  unit: {
    ...typography.body,
    color: colors.textMuted,
    fontSize: 11,
    marginTop: 2,
  },
});
