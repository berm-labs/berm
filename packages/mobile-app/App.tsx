import "react-native-get-random-values";
import React, { useEffect } from "react";
import { StatusBar } from "expo-status-bar";
import { SafeAreaProvider } from "react-native-safe-area-context";
import { NavigationContainer, DarkTheme } from "@react-navigation/native";
import { createBottomTabNavigator } from "@react-navigation/bottom-tabs";
import { HomeScreen } from "./src/screens/HomeScreen";
import { PositionsScreen } from "./src/screens/PositionsScreen";
import { AlertsScreen } from "./src/screens/AlertsScreen";
import { PoolsScreen } from "./src/screens/PoolsScreen";
import { BacktestScreen } from "./src/screens/BacktestScreen";
import { SettingsScreen } from "./src/screens/SettingsScreen";
import { TabIcon, type TabName } from "./src/components/TabIcon";
import { useStore } from "./src/state/useStore";
import { addNotificationListener } from "./src/lib/push";
import { colors } from "./src/theme/colors";

const Tab = createBottomTabNavigator();

const navTheme = {
  ...DarkTheme,
  colors: {
    ...DarkTheme.colors,
    primary: colors.glowCyan,
    background: colors.background,
    card: colors.breakwaterGrey,
    text: colors.fogWhite,
    border: colors.border,
    notification: colors.safetyAmber,
  },
};

export default function App() {
  const restore = useStore((s) => s.restore);
  const refreshWalletData = useStore((s) => s.refreshWalletData);
  const unread = useStore((s) => s.unreadCount());

  useEffect(() => {
    void restore();
    // Refresh wallet data when a push arrives so the UI reflects the new alert.
    const sub = addNotificationListener(() => {
      void refreshWalletData();
    });
    return () => sub.remove();
  }, [restore, refreshWalletData]);

  return (
    <SafeAreaProvider>
      <StatusBar style="light" />
      <NavigationContainer theme={navTheme}>
        <Tab.Navigator
          screenOptions={({ route }) => ({
            headerShown: false,
            tabBarActiveTintColor: colors.glowCyan,
            tabBarInactiveTintColor: colors.textMuted,
            tabBarStyle: {
              backgroundColor: colors.breakwaterGrey,
              borderTopColor: colors.border,
            },
            tabBarIcon: ({ focused }) => (
              <TabIcon name={route.name.toLowerCase() as TabName} focused={focused} />
            ),
          })}
        >
          <Tab.Screen name="Home" component={HomeScreen} />
          <Tab.Screen name="Positions" component={PositionsScreen} />
          <Tab.Screen
            name="Alerts"
            component={AlertsScreen}
            options={{ tabBarBadge: unread > 0 ? unread : undefined }}
          />
          <Tab.Screen name="Pools" component={PoolsScreen} />
          <Tab.Screen name="Backtest" component={BacktestScreen} />
          <Tab.Screen name="Settings" component={SettingsScreen} />
        </Tab.Navigator>
      </NavigationContainer>
    </SafeAreaProvider>
  );
}
