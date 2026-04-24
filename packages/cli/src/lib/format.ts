import { palette, theme } from "./theme.js";

export function usd(n: number): string {
  return `$${n.toLocaleString("en-US", { minimumFractionDigits: 2, maximumFractionDigits: 2 })}`;
}

export function num(n: number, digits = 2): string {
  return n.toLocaleString("en-US", { maximumFractionDigits: digits });
}

export function pct(fraction: number, digits = 1): string {
  return `${(fraction * 100).toFixed(digits)}%`;
}

export function kv(label: string, value: string, pad = 16): string {
  return `  ${theme.label(label.padEnd(pad))}${theme.value(value)}`;
}

export function heading(text: string): string {
  return `\n${palette.cyan.bold(text)}\n${palette.breakwater("-".repeat(text.length))}`;
}

// A compact, dependency-free fixed-width table renderer in the breakwater tone.
export function table(headers: string[], rows: string[][]): string {
  const widths = headers.map((h, i) =>
    Math.max(h.length, ...rows.map((r) => stripAnsi(r[i] ?? "").length)),
  );
  const head = headers
    .map((h, i) => theme.label(h.padEnd(widths[i])))
    .join("  ");
  const sep = palette.breakwater(widths.map((w) => "-".repeat(w)).join("  "));
  const body = rows
    .map((r) =>
      r
        .map((cell, i) => padAnsi(cell ?? "", widths[i]))
        .join("  "),
    )
    .join("\n");
  return `${head}\n${sep}\n${body}`;
}

// Horizontal meter bar, e.g. risk gauge.
export function meter(value0to100: number, width = 24): string {
  const filled = Math.round((value0to100 / 100) * width);
  return `${"#".repeat(filled)}${".".repeat(Math.max(0, width - filled))}`;
}

const ANSI = /\[[0-9;]*m/g;

function stripAnsi(s: string): string {
  return s.replace(ANSI, "");
}

function padAnsi(s: string, width: number): string {
  const visible = stripAnsi(s).length;
  const pad = Math.max(0, width - visible);
  return s + " ".repeat(pad);
}
