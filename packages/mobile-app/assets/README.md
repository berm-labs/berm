# App assets

Image assets referenced by `app.json`. Generate these from the marketing
breakwater renders (Phase 11) before building.

| File | Size | Purpose |
| --- | --- | --- |
| `icon.png` | 1024x1024 | iOS / Android app icon |
| `adaptive-icon.png` | 1024x1024 | Android adaptive foreground |
| `splash.png` | 1284x2778 | Launch splash (storm navy background) |
| `notification-icon.png` | 96x96 | Android notification small icon (monochrome) |

Palette: breakwater grey `#2A2A2A`, storm navy `#0A0E27`, glow cyan `#5BC0EB`.
Until the renders are dropped in, `expo prebuild` / `expo start` will warn about
missing assets; the JavaScript bundle and type checks are unaffected.
