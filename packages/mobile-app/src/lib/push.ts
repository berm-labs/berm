import { Platform } from "react-native";
import * as Notifications from "expo-notifications";
import * as Device from "expo-device";
import { colors } from "../theme/colors";
import { api } from "./api";

// Foreground presentation: show banner, play sound, set badge.
Notifications.setNotificationHandler({
  handleNotification: async () => ({
    shouldShowAlert: true,
    shouldPlaySound: true,
    shouldSetBadge: true,
  }),
});

export interface PushRegistration {
  token: string;
  platform: "ios" | "android";
}

// Requests notification permission and resolves the device push token.
// On Android the channel is created with the cyan accent. Backed by FCM
// (google-services.json) on Android and APNs on iOS via Expo's push service.
export async function registerForPush(): Promise<PushRegistration | null> {
  if (!Device.isDevice) return null;

  if (Platform.OS === "android") {
    await Notifications.setNotificationChannelAsync("cover-alerts", {
      name: "BERM Cover Alerts",
      importance: Notifications.AndroidImportance.HIGH,
      vibrationPattern: [0, 200, 100, 200],
      lightColor: colors.glowCyan,
      sound: "default",
    });
  }

  const settings = await Notifications.getPermissionsAsync();
  let granted = settings.granted;
  if (!granted) {
    const request = await Notifications.requestPermissionsAsync();
    granted = request.granted;
  }
  if (!granted) return null;

  const tokenResponse = await Notifications.getDevicePushTokenAsync();
  return {
    token: String(tokenResponse.data),
    platform: Platform.OS === "ios" ? "ios" : "android",
  };
}

// Registers the push token against a wallet so the backend can route alerts.
export async function subscribeWallet(wallet: string): Promise<boolean> {
  const registration = await registerForPush();
  if (!registration) return false;
  try {
    const res = await api.registerPushToken({
      wallet,
      token: registration.token,
      platform: registration.platform,
    });
    return res.ok;
  } catch {
    return false;
  }
}

export function addNotificationListener(
  handler: (notification: Notifications.Notification) => void,
): ReturnType<typeof Notifications.addNotificationReceivedListener> {
  return Notifications.addNotificationReceivedListener(handler);
}
