# BERM Alert wearables

Native widgets for the Apple Watch and Wear OS that surface the current cover
risk score, active cover count and unread alert badge at a glance.

Both widgets read a single shared snapshot whose shape is defined in
`src/lib/watch.ts` (`CoverSnapshot`). The React Native app persists this snapshot
after every wallet refresh; a thin native bridge mirrors it to each platform's
widget storage.

## Snapshot contract

```json
{
  "riskScore": 57,
  "band": "ROUGH",
  "activeCovers": 2,
  "unreadAlerts": 1,
  "updatedAt": "2026-06-14T00:00:00.000Z"
}
```

`band` is one of `CALM`, `CHOP`, `ROUGH`, `STORM`. The colour ramp matches the
in-app `RiskGauge` (cyan -> amber -> orange -> red).

## Apple Watch (`apple-watch/BermComplication.swift`)

1. Run `expo prebuild` to generate the native iOS project.
2. In Xcode add a **Widget Extension** target and include `BermComplication.swift`.
3. Enable the App Group `group.sh.berm.alert` on both the app and the extension.
4. Bridge `src/lib/watch.ts` output into the App Group `UserDefaults` under the
   key `berm.coverSnapshot` via an Expo config plugin (write on `persistSnapshot`).
5. Supported families: `accessoryCircular`, `accessoryInline`, `accessoryRectangular`.
6. Builds with the iOS app: `eas build --platform ios --profile production`.

## Wear OS (`wear-os/BermTileService.kt`, `wear-os/SnapshotStore.kt`)

1. Run `expo prebuild` to generate the native Android project.
2. Add a Wear OS module and register `BermTileService` in its `AndroidManifest.xml`
   with the `androidx.wear.tiles.action.BIND_TILE_PROVIDER` intent filter.
3. Add the dependencies `androidx.wear.tiles:tiles`,
   `androidx.wear.protolayout:protolayout` and `androidx.wear.protolayout:protolayout-material`.
4. Mirror the snapshot from the phone using the Wearable Data Layer
   (`DataClient`); on the watch, `SnapshotStore.write` persists it and
   `BermTileService` reads it via `SnapshotStore.read`.
5. Builds with the Android app: `eas build --platform android --profile production`.

## Colours

| Token | Hex |
| --- | --- |
| Breakwater grey | `#2A2A2A` |
| Glow cyan | `#5BC0EB` |
| Safety amber | `#FFD93D` |
| Warn | `#FFB347` |
| Danger | `#E5534B` |
