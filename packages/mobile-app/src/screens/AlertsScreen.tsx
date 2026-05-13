import React, { useCallback, useState } from "react";
import { StyleSheet, View } from "react-native";
import { ScreenContainer } from "../components/ScreenContainer";
import { SectionTitle } from "../components/Card";
import { AlertItem } from "../components/AlertItem";
import { EmptyState, ErrorState, Loading } from "../components/StateView";
import { useStore } from "../state/useStore";

export function AlertsScreen() {
  const wallet = useStore((s) => s.wallet);
  const alerts = useStore((s) => s.alerts);
  const refreshWalletData = useStore((s) => s.refreshWalletData);
  const [refreshing, setRefreshing] = useState(false);

  const onRefresh = useCallback(async () => {
    setRefreshing(true);
    await refreshWalletData();
    setRefreshing(false);
  }, [refreshWalletData]);

  return (
    <ScreenContainer refreshing={refreshing} onRefresh={onRefresh}>
      <SectionTitle>Alerts</SectionTitle>
      {!wallet ? (
        <EmptyState title="No wallet connected" hint="Connect a wallet to receive depeg, liquidation and claim alerts." />
      ) : alerts.loading && !alerts.data ? (
        <Loading label="Loading alerts" />
      ) : alerts.error && !alerts.data ? (
        <ErrorState message={alerts.error} />
      ) : alerts.data && alerts.data.length > 0 ? (
        <View style={styles.list}>
          {alerts.data.map((a) => (
            <AlertItem key={a.id} alert={a} />
          ))}
        </View>
      ) : (
        <EmptyState title="No alerts yet" hint="You will be notified when a trigger fires on your covers." />
      )}
    </ScreenContainer>
  );
}

const styles = StyleSheet.create({
  list: {
    gap: 10,
  },
});
