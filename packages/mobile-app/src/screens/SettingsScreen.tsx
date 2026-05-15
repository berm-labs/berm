import React, { useCallback, useState } from "react";
import { Linking, StyleSheet, Text, View } from "react-native";
import { ScreenContainer } from "../components/ScreenContainer";
import { Card, SectionTitle } from "../components/Card";
import { Button } from "../components/Button";
import { useStore } from "../state/useStore";
import { config } from "../lib/constants";
import { colors } from "../theme/colors";
import { typography } from "../theme/typography";
import { shortAddress } from "../lib/format";

export function SettingsScreen() {
  const wallet = useStore((s) => s.wallet);
  const connecting = useStore((s) => s.connecting);
  const connect = useStore((s) => s.connect);
  const disconnect = useStore((s) => s.disconnect);
  const [error, setError] = useState<string | null>(null);

  const onConnect = useCallback(async () => {
    setError(null);
    try {
      await connect();
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    }
  }, [connect]);

  return (
    <ScreenContainer>
      <SectionTitle>Wallet</SectionTitle>
      <Card>
        {wallet ? (
          <>
            <Row label="Connected" value={shortAddress(wallet)} />
            <Button label="Disconnect" variant="ghost" onPress={() => void disconnect()} />
          </>
        ) : (
          <>
            <Text style={styles.copy}>No wallet connected.</Text>
            <Button label={connecting ? "Connecting" : "Connect wallet"} onPress={onConnect} loading={connecting} />
            {error ? <Text style={styles.error}>{error}</Text> : null}
          </>
        )}
      </Card>

      <SectionTitle>Network</SectionTitle>
      <Card>
        <Row label="Cluster" value={config.cluster} />
        <Row label="RPC" value={config.rpcUrl.replace(/^https?:\/\//, "")} />
        <Row label="API" value={config.apiUrl.replace(/^https?:\/\//, "")} />
      </Card>

      <SectionTitle>About</SectionTitle>
      <Card>
        <Text style={styles.copy}>
          BERM Alert tracks parametric cover on Solana and notifies you when a depeg, liquidation,
          claim trigger or new risk affects your positions.
        </Text>
        <Button label="Open berm.sh" variant="ghost" onPress={() => void Linking.openURL(config.siteUrl)} />
      </Card>
    </ScreenContainer>
  );
}

function Row({ label, value }: { label: string; value: string }) {
  return (
    <View style={styles.row}>
      <Text style={styles.rowLabel}>{label}</Text>
      <Text style={styles.rowValue}>{value}</Text>
    </View>
  );
}

const styles = StyleSheet.create({
  row: {
    flexDirection: "row",
    justifyContent: "space-between",
    alignItems: "center",
    paddingVertical: 4,
  },
  rowLabel: {
    ...typography.body,
    color: colors.textMuted,
    fontSize: 13,
  },
  rowValue: {
    ...typography.mono,
    color: colors.textPrimary,
    fontSize: 13,
    flexShrink: 1,
    textAlign: "right",
    marginLeft: 12,
  },
  copy: {
    ...typography.body,
    color: colors.textMuted,
    fontSize: 14,
  },
  error: {
    ...typography.body,
    color: colors.danger,
    fontSize: 13,
    marginTop: 8,
  },
});
