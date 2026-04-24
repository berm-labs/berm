import chalk from "chalk";

// Storm Breakwater palette mapped to terminal-safe truecolor.
// Dark grey base, glowing cyan primary, safety amber, accent gold.
export const palette = {
  breakwater: chalk.hex("#5A6B7C"), // wave slate, structural text
  cyan: chalk.hex("#5BC0EB"), // glowing cyan, primary
  amber: chalk.hex("#FFD93D"), // safety amber, active cover
  gold: chalk.hex("#D4AF37"), // accent gold, token / rewards
  fog: chalk.hex("#E8EAED"), // fog white, body text
  glow: chalk.hex("#F0EAD6"), // breakwater white glow, emphasis
  storm: chalk.hex("#0A0E27"), // storm navy (rarely used on dark terms)
};

export const theme = {
  // Semantic helpers
  title: (s: string) => palette.cyan.bold(s),
  subtitle: (s: string) => palette.breakwater(s),
  label: (s: string) => palette.breakwater(s),
  value: (s: string) => palette.fog(s),
  accent: (s: string) => palette.amber(s),
  token: (s: string) => palette.gold(s),
  ok: (s: string) => palette.cyan(s),
  safe: (s: string) => palette.amber(s),
  warn: (s: string) => chalk.hex("#FFB347")(s),
  danger: (s: string) => chalk.hex("#E5534B")(s),
  muted: (s: string) => chalk.hex("#6B7280")(s),
  dim: (s: string) => chalk.dim(s),
};

// Severity colour ramp for risk scores (0 calm -> 100 storm).
export function riskColor(score: number): (s: string) => string {
  if (score >= 75) return theme.danger;
  if (score >= 50) return theme.warn;
  if (score >= 25) return theme.safe;
  return theme.ok;
}

export function riskBand(score: number): string {
  if (score >= 75) return "STORM";
  if (score >= 50) return "ROUGH";
  if (score >= 25) return "CHOP";
  return "CALM";
}

// Minimal wordmark. Not a diagram, a CLI masthead.
export function masthead(version: string): string {
  const bar = palette.breakwater("============================");
  const name = palette.cyan.bold("BERM");
  const tag = palette.fog("Solana cover protocol");
  const v = theme.muted(`v${version}`);
  return `${bar}\n  ${name}  ${tag}  ${v}\n  ${palette.glow("Break the wave.")}\n${bar}`;
}
