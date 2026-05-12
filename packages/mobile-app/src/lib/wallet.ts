import { transact } from "@solana-mobile/mobile-wallet-adapter-protocol-web3js";
import type { Chain } from "@solana-mobile/mobile-wallet-adapter-protocol";
import { PublicKey } from "@solana/web3.js";
import AsyncStorage from "@react-native-async-storage/async-storage";
import { APP_IDENTITY, config } from "./constants";

const AUTH_TOKEN_KEY = "berm.mwa.authToken";
const ADDRESS_KEY = "berm.mwa.address";

const CHAIN: Chain =
  config.cluster === "mainnet-beta"
    ? "solana:mainnet"
    : config.cluster === "devnet"
      ? "solana:devnet"
      : "solana:testnet";

export interface ConnectedWallet {
  address: string;
  authToken: string;
}

// Connects via the Solana Mobile Wallet Adapter. The auth token is persisted so
// subsequent sessions can reauthorize silently. Returns the base58 address.
export async function connectWallet(): Promise<ConnectedWallet> {
  const previousAuthToken = (await AsyncStorage.getItem(AUTH_TOKEN_KEY)) ?? undefined;

  const result = await transact(async (wallet) => {
    const auth = await wallet.authorize({
      chain: CHAIN,
      identity: APP_IDENTITY,
      auth_token: previousAuthToken,
    });
    const account = auth.accounts[0];
    const address = new PublicKey(toUint8Array(account.address)).toBase58();
    return { address, authToken: auth.auth_token };
  });

  await AsyncStorage.multiSet([
    [AUTH_TOKEN_KEY, result.authToken],
    [ADDRESS_KEY, result.address],
  ]);
  return result;
}

export async function disconnectWallet(): Promise<void> {
  const authToken = (await AsyncStorage.getItem(AUTH_TOKEN_KEY)) ?? undefined;
  if (authToken) {
    try {
      await transact(async (wallet) => {
        await wallet.deauthorize({ auth_token: authToken });
      });
    } catch {
      // Wallet app may be unavailable; clearing local state is sufficient.
    }
  }
  await AsyncStorage.multiRemove([AUTH_TOKEN_KEY, ADDRESS_KEY]);
}

export async function getStoredAddress(): Promise<string | null> {
  return AsyncStorage.getItem(ADDRESS_KEY);
}

// Mobile Wallet Adapter returns base64 addresses; decode without Buffer for RN.
function toUint8Array(base64: string): Uint8Array {
  const binary = globalThis.atob(base64);
  const bytes = new Uint8Array(binary.length);
  for (let i = 0; i < binary.length; i++) bytes[i] = binary.charCodeAt(i);
  return bytes;
}
