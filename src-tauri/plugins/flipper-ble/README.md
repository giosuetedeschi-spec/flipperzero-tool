# flipper-ble plugin

The platform-specific half of the BLE link. On iOS it is the **only** way to
reach a Flipper — USB serial requires MFi — and on Android it is the default,
with USB-OTG as the faster alternative when a cable is attached.

## Where the logic lives

Almost none of it is here. `src-tauri/src/transport/ble.rs` owns the chunking,
flow-control waiting and buffering, and is covered by 15 unit tests against a
fake link. These two files only have to be correct GATT clients: they move bytes
and report two numbers.

That boundary is deliberate. Everything testable without hardware sits on the
Rust side of the `BleLink` trait; only what genuinely needs a radio sits here.

| | |
|---|---|
| `android/FlipperBleLink.kt` | `BluetoothGatt` client |
| `ios/FlipperBleLink.swift` | CoreBluetooth central |

**Neither is verified in CI.** There is no Android device or emulator in the
build, and iOS needs macOS with Xcode. Both are exercised only on real hardware.

## Wiring them in

`tauri android init` and `tauri ios init` generate the native shells under
`src-tauri/gen/`. Drop these sources into the generated projects and expose the
five `BleLink` methods across the Tauri bridge:
`write_chunk`, `drain_notifications`, `mtu_payload`, `free_device_buffer`,
`is_connected`.

### Android permissions

`AndroidManifest.xml` needs `BLUETOOTH_SCAN` and `BLUETOOTH_CONNECT` (API 31+),
plus `ACCESS_FINE_LOCATION` for scanning on older releases. Background scanning
additionally needs `FOREGROUND_SERVICE` and
`FOREGROUND_SERVICE_CONNECTED_DEVICE`, and the service must show a persistent
notification.

### iOS

`Info.plist` needs `NSBluetoothAlwaysUsageDescription`, and
`UIBackgroundModes: [bluetooth-central]` for background operation. iOS restricts
background BLE heavily — expect a slower, best-effort cadence there, not
parity with Android, and say so in the UI rather than letting the difference
look like a bug.

## Things that will bite

**The GATT UUIDs.** They are gathered at the top of `transport/ble.rs` and
duplicated in both native files. If the connection succeeds but no notification
ever arrives, check them against the target firmware first — that is by far the
most likely failure.

**The CCC descriptor on Android.** `setCharacteristicNotification` alone is not
enough; without writing the client-characteristic-config descriptor the
peripheral never sends anything, and the link looks connected but permanently
silent.

**MTU before service discovery.** The Android side requests the MTU on connect
and only discovers services once it settles, so characteristics are used at
their final size from the first write. At the 20-byte default every transfer is
roughly ten times slower than it needs to be.

**Write without response.** Both sides use it deliberately; the Flipper's own
flow-control characteristic replaces the per-write acknowledgement that would
otherwise pace the link. Ignoring flow control does not merely slow things down —
packets are dropped silently and the RPC stream desynchronises with no error
anywhere.
