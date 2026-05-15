// Storm Breakwater palette. Dark grey base, glowing cyan primary, accent gold.
export const colors = {
  breakwaterGrey: "#2A2A2A",
  waveSlate: "#5A6B7C",
  stormNavy: "#0A0E27",
  glowCyan: "#5BC0EB",
  safetyAmber: "#FFD93D",
  whiteGlow: "#F0EAD6",
  accentGold: "#D4AF37",
  fogWhite: "#E8EAED",

  // Derived UI surfaces
  background: "#0A0E27",
  surface: "#1A1F33",
  surfaceRaised: "#232A42",
  border: "#2E3650",
  textPrimary: "#E8EAED",
  textMuted: "#8A93A8",
  danger: "#E5534B",
  warn: "#FFB347",
  ok: "#5BC0EB",
} as const;

// Risk band colour ramp (0 calm -> 100 storm).
export function riskColor(score: number): string {
  if (score >= 75) return colors.danger;
  if (score >= 50) return colors.warn;
  if (score >= 25) return colors.safetyAmber;
  return colors.glowCyan;
}

export function riskBand(score: number): "CALM" | "CHOP" | "ROUGH" | "STORM" {
  if (score >= 75) return "STORM";
  if (score >= 50) return "ROUGH";
  if (score >= 25) return "CHOP";
  return "CALM";
}
