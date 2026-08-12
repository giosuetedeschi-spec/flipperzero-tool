# What still needs real hardware

Everything in this file is written and unverified. It could not be tested in the
environment it was built in — no Android SDK, no macOS, no Flipper — so the
logic that *could* be tested was deliberately pushed above the hardware boundary
and covered there. What remains below the line is genuinely device-dependent.

This is the list to work through with a Flipper in hand.

---

## 1. Generate the native shells

Nothing installable exists until this runs. It is the single step between the
repository and an APK.

```bash
npx tauri android init     # needs ANDROID_HOME and an NDK
npx tauri ios init         # macOS with Xcode only
```

Then add the permissions listed in
[`src-tauri/plugins/flipper-ble/README.md`](../src-tauri/plugins/flipper-ble/README.md)
to the generated `AndroidManifest.xml` and `Info.plist`.

**Expected outcome:** `npx tauri android build --apk --debug` produces an APK
that installs and opens on the Radar screen.

---

## 2. BLE connection

The chunking, flow-control waiting and buffering are unit tested against a fake
link (15 tests). The GATT client underneath is not.

Drop `FlipperBleLink.kt` and `FlipperBleLink.swift` into the generated projects
and bridge the five `BleLink` methods.

**Check, in order — each failure has a specific first suspect:**

1. **The Flipper is discovered when scanning.** If not, the service UUID is
   wrong. All the GATT UUIDs are gathered at the top of
   `src-tauri/src/transport/ble.rs`.
2. **Connection succeeds and notifications arrive.** Connected but permanently
   silent, on Android, is almost always the CCC descriptor:
   `setCharacteristicNotification` alone does not make the peripheral send
   anything.
3. **The MTU rises above 20.** The Android side requests it before service
   discovery. At the 20-byte default every transfer is roughly ten times slower
   than it needs to be.
4. **A large file transfers intact.** This is where flow control shows: if the
   Flipper's buffer is being overrun the packets vanish silently and the RPC
   stream desynchronises with no error anywhere. Read a `.fap` and compare it
   byte for byte with the same file over USB.

---

## 3. The Signal Radar FAP

```bash
cd flipper-fap/signal_radar
ufbt launch
```

uFBT downloads the Flipper SDK on first run, which is why this was never built
in CI.

**Check:**

1. **It builds.** API breakage between firmware versions is the likely failure;
   the FAP is written against the official SDK.
2. **`Hello` answers with protocol version 1.** The wire format is verified from
   the Rust side (19 tests), so a mismatch here means the C encoder disagrees
   with `protocol.h` rather than with the decoder.
3. **Sub-GHz sweeping reports something near a transmitting remote and nothing
   in a quiet room.** The noise floor is `SUBGHZ_RSSI_FLOOR_DBM` in
   `signal_radar.c`; if everything reports constantly, raise it. A Radar that
   always shows signals teaches the user it means nothing.
4. **The app stays responsive while scanning.** The scan loop runs on a worker
   thread precisely so the RPC thread never blocks; a blocked RPC thread looks
   identical to a dropped connection from the phone.

---

## 4. Deploying the FAP from the phone

Not implemented. The pieces it needs are in place — binary round-tripping works
end to end, `Storage.Write` is exercised against the mock — but nothing yet
writes the `.fap` and starts it.

The intended flow is in [`flipper-fap/README.md`](../flipper-fap/README.md):
write to `/ext/apps/Tools/`, start with `App.StartRequest`, send `Hello`,
compare versions, redeploy on mismatch.

---

## 5. Transmitting

Also not implemented, and it fails honestly today: Replay and Emulate return
"the on-device app cannot transmit yet" rather than doing nothing quietly.

Two guards are already in place and worth confirming once transmission exists:

- A **rolling-code** remote is refused even if a caller asks directly, not just
  hidden in the UI.
- An **out-of-band frequency** is refused before being sent, because stock
  firmware declines it silently — a replay that appears to work and changes
  nothing is the most confusing failure this app can produce.

---

## 6. Things that will differ between platforms

Worth setting expectations rather than treating as bugs:

- **Background scanning.** Android needs a foreground service with a persistent
  notification. iOS restricts background BLE heavily, so its cadence will be
  slower and best-effort. This difference should be stated in the UI, not left
  to look like a fault.
- **BLE throughput.** A few kilobytes per second. Transferring a `.fap` will
  take visible seconds; on Android, prefer USB-OTG when a cable is attached.
- **Bluetooth scanning.** Impossible while the phone holds the BLE link. This is
  physics, not a bug, and the app says so.

---

## Not hardware, but also outstanding

**Replace the hand-written protobuf.** `src-tauri/proto/flipper.proto` is a
hand-maintained subset that already diverges from upstream. It needs the real
`flipperzero-protobuf` as a submodule plus `prost-build` in `build.rs`. This was
not done here because the environment could not reach the upstream repository,
and rewriting the schema from memory would have reproduced exactly the problem
it was meant to solve.
