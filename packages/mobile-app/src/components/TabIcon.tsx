import React from "react";
import Svg, { Circle, Path, Rect } from "react-native-svg";
import { colors } from "../theme/colors";

export type TabName = "home" | "positions" | "alerts" | "pools" | "backtest" | "settings";

interface Props {
  name: TabName;
  focused: boolean;
}

// Minimal line icons drawn with SVG primitives. No emoji, no icon font.
export function TabIcon({ name, focused }: Props) {
  const color = focused ? colors.glowCyan : colors.textMuted;
  const stroke = 1.8;
  switch (name) {
    case "home":
      return (
        <Svg width={24} height={24} viewBox="0 0 24 24">
          <Path d="M4 11 12 5l8 6v7a1 1 0 0 1-1 1h-4v-5h-6v5H5a1 1 0 0 1-1-1z" fill="none" stroke={color} strokeWidth={stroke} strokeLinejoin="round" />
        </Svg>
      );
    case "positions":
      return (
        <Svg width={24} height={24} viewBox="0 0 24 24">
          <Rect x={4} y={5} width={16} height={14} rx={2} fill="none" stroke={color} strokeWidth={stroke} />
          <Path d="M8 10h8M8 14h5" stroke={color} strokeWidth={stroke} strokeLinecap="round" />
        </Svg>
      );
    case "alerts":
      return (
        <Svg width={24} height={24} viewBox="0 0 24 24">
          <Path d="M6 10a6 6 0 0 1 12 0c0 5 2 6 2 6H4s2-1 2-6z" fill="none" stroke={color} strokeWidth={stroke} strokeLinejoin="round" />
          <Path d="M10 19a2 2 0 0 0 4 0" fill="none" stroke={color} strokeWidth={stroke} strokeLinecap="round" />
        </Svg>
      );
    case "pools":
      return (
        <Svg width={24} height={24} viewBox="0 0 24 24">
          <Rect x={4} y={13} width={4} height={6} fill="none" stroke={color} strokeWidth={stroke} />
          <Rect x={10} y={9} width={4} height={10} fill="none" stroke={color} strokeWidth={stroke} />
          <Rect x={16} y={5} width={4} height={14} fill="none" stroke={color} strokeWidth={stroke} />
        </Svg>
      );
    case "backtest":
      return (
        <Svg width={24} height={24} viewBox="0 0 24 24">
          <Path d="M4 19V5M4 19h16" fill="none" stroke={color} strokeWidth={stroke} strokeLinecap="round" />
          <Path d="M7 15l4-5 3 3 4-6" fill="none" stroke={color} strokeWidth={stroke} strokeLinecap="round" strokeLinejoin="round" />
        </Svg>
      );
    case "settings":
    default:
      return (
        <Svg width={24} height={24} viewBox="0 0 24 24">
          <Circle cx={12} cy={12} r={3} fill="none" stroke={color} strokeWidth={stroke} />
          <Path d="M12 3v3M12 18v3M3 12h3M18 12h3M5.6 5.6l2.1 2.1M16.3 16.3l2.1 2.1M18.4 5.6l-2.1 2.1M7.7 16.3l-2.1 2.1" stroke={color} strokeWidth={stroke} strokeLinecap="round" />
        </Svg>
      );
  }
}
