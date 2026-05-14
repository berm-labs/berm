import React, { useCallback, useState } from "react";
import { ScreenContainer } from "../components/ScreenContainer";
import { SectionTitle } from "../components/Card";
import { CoverCard } from "../components/CoverCard";
import { EmptyState, ErrorState, Loading } from "../components/StateView";
import { useStore } from "../state/useStore";

export function PositionsScreen() {
  const wallet = useStore((s) => s.wallet);
  const positions = useStore((s) => s.positions);
  const refreshWalletData = useStore((s) => s.refreshWalletData);
  const [refreshing, setRefreshing] = useState(false);

  const onRefresh = useCallback(async () => {
    setRefreshing(true);
    await refreshWalletData();
    setRefreshing(false);
  }, [refreshWalletData]);

  return (
    <ScreenContainer refreshing={refreshing} onRefresh={onRefresh}>
      <SectionTitle>Your covers</SectionTitle>
      {!wallet ? (
        <EmptyState title="No wallet connected" hint="Connect a wallet on the Home tab to view your covers." />
      ) : positions.loading && !positions.data ? (
        <Loading label="Loading covers" />
      ) : positions.error && !positions.data ? (
        <ErrorState message={positions.error} />
      ) : positions.data && positions.data.length > 0 ? (
        positions.data.map((p) => <CoverCard key={p.id} position={p} />)
      ) : (
        <EmptyState title="No active covers" hint="Design a cover on berm.sh to protect this wallet." />
      )}
    </ScreenContainer>
  );
}
