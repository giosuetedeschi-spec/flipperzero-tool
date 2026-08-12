# FlipperZero Tool

Desktop and mobile app for the Flipper Zero: a **Signal Radar** that shows what
the device's radios can hear, plus a file manager, format viewers, and plugin
development tools.

## Signal Radar

An interactive diagram of the Flipper. Tap a chip to see what it is picking up,
how many distinct signals it has found, and what you can do with each — in plain
language, for someone who has never heard of Sub-GHz or a rolling code.

Three behaviours are worth knowing up front, because they are deliberate:

- **The count is distinct signals, not sightings.** Walk past the same gate ten
  times and it stays one signal with a higher sighting count.
- **Rolling-code remotes are not offered a Replay button.** The code changes on
  every press, so replaying does nothing; offering it would teach a wrong model
  of how your own gate works.
- **Bluetooth reports itself unavailable over BLE.** While your phone occupies
  the Flipper's radio it cannot scan for other devices, and an empty list would
  read as "nothing nearby".

## Platforms

| | Transport | Status |
|---|---|---|
| Desktop (Linux/macOS/Windows) | USB CDC serial | Working |
| Android | BLE, USB-OTG | Rust verified in CI; native shell not yet generated |
| iOS | BLE only (USB needs MFi) | Rust verified in CI; native shell not yet generated |

`serialport` is desktop-only and cannot build for mobile, so it lives behind
`#[cfg(desktop)]`. CI asserts it stays out of both mobile dependency trees.

## Tech stack

- **Shell**: Tauri v2 — one Rust core for desktop and mobile
- **Frontend**: React 19 + TypeScript + Tailwind v4, bilingual IT/EN
- **Backend**: Rust — transport abstraction, length-delimited protobuf RPC,
  SQLite (rusqlite) for the file cache and signal history
- **On-device**: `flipper-fap/signal_radar`, a C app built with uFBT that does
  the actual scanning

## Setup

### Prerequisites

- Rust 1.85+ (the crate is edition 2024)
- Node.js 20+
- Linux also needs: `libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev
  patchelf libudev-dev`

### Run

```bash
cd frontend && npm install && npm run build
npx tauri dev
```

### Develop without a Flipper

A mock device speaks the real RPC protocol over TCP, so the whole stack —
transport, framing, session, parsers — runs against it with no hardware:

```bash
cd src-tauri
cargo run --bin flipper_mock          # listens on 127.0.0.1:9999
```

The app also has a **simulation mode** in Settings, which fills the Radar with
example signals for exploring the interface.

## Mobile builds

The Rust core is verified for both mobile targets in CI, but the native project
shells are not in the repository — generating them needs an Android SDK and, for
iOS, macOS with Xcode:

```bash
npx tauri android init     # creates src-tauri/gen/android
npx tauri ios init         # creates src-tauri/gen/apple, macOS only

npx tauri android build --apk --debug
npx tauri ios build --debug
```

Android needs `BLUETOOTH_SCAN`, `BLUETOOTH_CONNECT`, `ACCESS_FINE_LOCATION` and
the USB host feature in its manifest; iOS needs
`NSBluetoothAlwaysUsageDescription`. See
[`src-tauri/plugins/flipper-ble/README.md`](src-tauri/plugins/flipper-ble/README.md).

## Tests

```bash
cd src-tauri && cargo test          # 276 tests
cd frontend  && npm test            # 96 tests
```

CI additionally cross-compiles for `aarch64-linux-android` and
`aarch64-apple-ios`, which is what catches a desktop-only dependency leaking
back into shared code.

## Documentation

- [`docs/MOBILE-PORT-PLAN.md`](docs/MOBILE-PORT-PLAN.md) — architecture and phase status
- [`docs/HARDWARE-CHECKLIST.md`](docs/HARDWARE-CHECKLIST.md) — what still needs a real device
- [`flipper-fap/README.md`](flipper-fap/README.md) — the on-device app
- [`src-tauri/plugins/flipper-ble/README.md`](src-tauri/plugins/flipper-ble/README.md) — the BLE bridge

## Licence

MIT.
