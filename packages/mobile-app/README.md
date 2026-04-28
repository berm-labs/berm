# BERM Alert

Real-time Solana cover status and parametric risk alerts for iOS and Android.
Part of **Berm**, the parametric DeFi cover protocol on Solana.

> Break the wave.

BERM Alert connects a Solana wallet via the Mobile Wallet Adapter, reads the
wallet's live position risk directly from the chain, tracks active covers, and
delivers push alerts the moment a parametric trigger fires.

> **Currently runs on Solana devnet. Mainnet pending.** The default cluster is
> `devnet`; the public devnet RPC is used for all on-chain reads.

- Website: https://berm.sh
- X: https://x.com/berm_sh
- Source: https://github.com/berm-labs/berm

## Features

- **Live risk score** computed on device from on-chain holdings (depeg, slashing
  and concentration sub-scores), identical to the `berm-cli` scoring curve.
- **Cover positions** with amount, premium, risk and expiry.
- **Push alerts** on the "BERM Cover Alerts" channel for four event kinds: depeg
  detected, liquidation imminent, claim auto-triggered, new risk detected.
- **Cover pools** overview with TVL, utilization, APR and trigger counts.
- **Backtest** of cover payouts against documented historical loss events
  (Mango exploit, USDC depeg, mSOL depeg) -- deterministic simulations.
- **Apple Watch complication and Wear OS tile** showing risk and active covers
  at a glance (see [`watch/`](./watch/README.md)).
- **Solana Mobile Wallet Adapter** sign-in with persistent reauthorization.

## Screens

Home, Positions, Alerts, Pools, Backtest, Settings.

## Stack

- Expo SDK 51 / React Native 0.74
- `@solana-mobile/mobile-wallet-adapter-protocol-web3js`
- `@solana/web3.js` for on-chain reads (public RPC only)
- `expo-notifications` over FCM (Android) and APNs (iOS)
- `@react-navigation` bottom tabs
- `zustand` state, `react-native-svg` risk gauge

## Configuration

Public configuration lives in `app.json` under `expo.extra` and is read in
`src/lib/constants.ts`. No secrets are bundled; only public RPC is used.

| Key | Default |
| --- | --- |
| `apiUrl` | `https://api.berm.sh` |
| `rpcUrl` | `https://api.devnet.solana.com` |
| `cluster` | `devnet` |
| `siteUrl` | `https://berm.sh` |

Deep links: the app registers `https://berm.sh` (iOS associated domains +
Android App Links) and the `bermalert://` custom scheme.

## Develop

```bash
npm install
npm run typecheck
npm start            # Expo dev server (Expo Go or a dev client)
```

The Mobile Wallet Adapter requires a custom dev client or a release build (it is
not available in Expo Go). Use `expo run:android` / `expo run:ios` for a full
native run.

## Build

Builds use EAS. Profiles are defined in `eas.json`.

```bash
# Android App Bundle (.aab) for Play / dApp Store
npm run build:android

# Android APK (.apk) for direct install / internal testing
npm run build:android:apk

# iOS (.ipa) - requires an Apple Developer account
npm run build:ios
```

Local native builds (after `expo prebuild`):

```bash
expo prebuild
# Android .apk
cd android && ./gradlew assembleRelease     # output: android/app/build/outputs/apk/release/
# iOS .ipa
cd ios && xcodebuild -scheme bermalert -configuration Release archive
```

### Build prerequisites

- **Android push (FCM):** copy `google-services.json.example` to
  `google-services.json` with your Firebase project values.
- **iOS push (APNs):** configure the push capability and an APNs key in your
  Apple Developer account; EAS manages the provisioning profile.
- **App icons / splash:** add the images listed in [`assets/README.md`](./assets/README.md).

> The `.apk` / `.ipa` binaries are produced by the commands above. They are not
> committed to the repo; CI / EAS uploads the artifacts. The build manifest and
> step-by-step instructions are kept with the project deliverables.

## Solana Mobile dApp Store

Publishing config is in [`dapp-store/config.yaml`](./dapp-store/config.yaml).

```bash
npx dapp-store create publisher -k <keypair> -u <rpc>
npx dapp-store create app -k <keypair> -u <rpc>
npx dapp-store create release -k <keypair> -u <rpc> -b <android-sdk-build-tools>
npx dapp-store publish submit -k <keypair> -u <rpc> --requestor-is-authorized
```

## Project layout

```
mobile-app/
  App.tsx                 navigation + push wiring
  app.json                Expo config (FCM, APNs, extra)
  eas.json                build profiles
  dapp-store/config.yaml  Solana Mobile dApp Store manifest
  src/
    lib/                  api, risk, wallet (MWA), push, watch snapshot, backtest
    state/useStore.ts     zustand store
    components/           gauge, cards, pills, buttons, tab icons
    screens/              Home, Positions, Alerts, Pools, Backtest, Settings
    theme/                Storm Breakwater palette + typography
  watch/
    apple-watch/          WidgetKit complication (SwiftUI)
    wear-os/              Wear OS tile (Kotlin)
```

## License

MIT
