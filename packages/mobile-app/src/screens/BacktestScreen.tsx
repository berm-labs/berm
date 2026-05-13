import React, { useMemo, useState } from "react";
import { StyleSheet, Text, TextInput, View } from "react-native";
import { ScreenContainer } from "../components/ScreenContainer";
import { Card, SectionTitle } from "../components/Card";
import { StatusPill } from "../components/StatusPill";
import { runBacktest } from "../lib/backtest";
import { COVER_TYPE_META, type CoverType } from "../lib/constants";
import { colors } from "../theme/colors";
import { typography } from "../theme/typography";
import { pct, usd } from "../lib/format";

export function BacktestScreen() {
  const [amountText, setAmountText] = useState("1000");

  const amount = useMemo(() => {
    const n = Number(amountText.replace(/[^0-9.]/g, ""));
    return Number.isFinite(n) && n > 0 ? n : 0;
  }, [amountText]);

  const results = useMemo(() => runBacktest(amount), [amount]);
  const total = results.reduce((a, r) => a + r.payoutUsd, 0);

  return (
    <ScreenContainer>
      <SectionTitle>Backtest</SectionTitle>
      <Card>
        <Text style={styles.copy}>
          See what a cover of this size would have paid out across documented historical
          loss events. These are deterministic simulations, not live claims.
        </Text>
        <View style={styles.inputRow}>
          <Text style={styles.inputLabel}>Cover amount (USD)</Text>
          <TextInput
            value={amountText}
            onChangeText={setAmountText}
            keyboardType="numeric"
            placeholder="1000"
            placeholderTextColor={colors.textMuted}
            style={styles.input}
          />
        </View>
        <View style={styles.totalRow}>
          <Text style={styles.totalLabel}>Total simulated payout</Text>
          <Text style={styles.totalValue}>{usd(total)}</Text>
        </View>
      </Card>

      {results.map((r) => {
        const meta = COVER_TYPE_META[r.coverType as CoverType];
        return (
          <Card key={r.id}>
            <View style={styles.headerRow}>
              <Text style={styles.title}>{r.title}</Text>
              <StatusPill label={r.date} tone="muted" />
            </View>
            <Text style={styles.cover}>{meta.label}</Text>
            <View style={styles.metrics}>
              <Metric label="Loss depth" value={pct(r.lossFraction)} />
              <Metric label="Payout" value={usd(r.payoutUsd)} accent />
            </View>
            <Text style={styles.summary}>{r.summary}</Text>
            <Text style={styles.reference}>Reference: {r.reference}</Text>
          </Card>
        );
      })}
    </ScreenContainer>
  );
}

function Metric({ label, value, accent }: { label: string; value: string; accent?: boolean }) {
  return (
    <View style={styles.metric}>
      <Text style={styles.metricLabel}>{label}</Text>
      <Text style={[styles.metricValue, accent && { color: colors.accentGold }]}>{value}</Text>
    </View>
  );
}

const styles = StyleSheet.create({
  copy: {
    ...typography.body,
    color: colors.textMuted,
    fontSize: 13,
  },
  inputRow: {
    gap: 6,
    marginTop: 4,
  },
  inputLabel: {
    ...typography.body,
    color: colors.textMuted,
    fontSize: 12,
  },
  input: {
    ...typography.mono,
    color: colors.textPrimary,
    fontSize: 18,
    borderWidth: 1,
    borderColor: colors.border,
    borderRadius: 10,
    paddingHorizontal: 12,
    paddingVertical: 10,
    backgroundColor: colors.surfaceRaised,
  },
  totalRow: {
    flexDirection: "row",
    justifyContent: "space-between",
    alignItems: "center",
    marginTop: 8,
  },
  totalLabel: {
    ...typography.body,
    color: colors.textMuted,
    fontSize: 13,
  },
  totalValue: {
    ...typography.display,
    color: colors.accentGold,
    fontSize: 20,
  },
  headerRow: {
    flexDirection: "row",
    justifyContent: "space-between",
    alignItems: "center",
  },
  title: {
    ...typography.title,
    color: colors.fogWhite,
    fontSize: 16,
  },
  cover: {
    ...typography.mono,
    color: colors.glowCyan,
    fontSize: 13,
  },
  metrics: {
    flexDirection: "row",
    gap: 24,
    marginTop: 4,
  },
  metric: {
    gap: 2,
  },
  metricLabel: {
    ...typography.body,
    color: colors.textMuted,
    fontSize: 11,
  },
  metricValue: {
    ...typography.mono,
    color: colors.textPrimary,
    fontSize: 15,
  },
  summary: {
    ...typography.body,
    color: colors.textMuted,
    fontSize: 13,
  },
  reference: {
    ...typography.body,
    color: colors.waveSlate,
    fontSize: 11,
  },
});
