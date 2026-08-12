# Signal Radar FAP

The on-device half of the mobile app's live view.

Stock Flipper firmware exposes no continuous scan over RPC — the protocol covers
storage, system, GUI, app launching and GPIO, and nothing that answers "what can
the radios hear right now". This app fills that gap: it owns the scanning loop
and streams each detection to the phone over the RPC data-exchange channel.

## Building

```
cd flipper-fap/signal_radar
ufbt              # build
ufbt launch       # build, upload and run on a connected Flipper
```

uFBT downloads the Flipper SDK on first run. A machine that cannot reach
`update.flipperzero.one` cannot build this, which is why the FAP is absent from
CI.

## How the two halves are kept in agreement

`protocol.h` is the single source of truth for the wire format. The Rust decoder
in `src-tauri/src/signals/fap_protocol.rs` mirrors it field for field, and **its
tests are the only automated check that the two agree** — they cover the header,
every event type, partial and truncated messages, absurd lengths, the RSSI
sentinel, and version mismatch.

If you change the layout, bump `SIGNAL_RADAR_PROTOCOL_VERSION` in both files. The
phone compares versions at connect and redeploys the FAP on mismatch, so a stale
app never misreads a new layout. `Hello` is answered regardless of version —
it is how the mismatch is discovered in the first place.

## Deployment from the phone

1. The app writes `signal_radar.fap` to `/ext/apps/Tools/` with `Storage.Write`
   over RPC. This needs binary round-tripping, which is why the missing base64
   decoder had to be fixed first.
2. It starts the app with `App.StartRequest`.
3. It sends `Hello` and compares the reported version with its own.
4. On mismatch, it rewrites the `.fap` and restarts it.

## What still needs a real device

Only Sub-GHz sweeping is implemented, and as a **detector rather than a
decoder**: it reports which frequencies carry more than noise, with the
`Decoded` flag clear. That distinction is deliberate — the Rust side treats an
undecoded burst as having an unknown code scheme and will not offer to replay
it, because an undecoded capture might be a rolling code.

NFC, LF RFID, iButton and IR follow the same shape: poll the chip, and on a hit
call `signal_radar_send_signal` with the identifying bytes as `data` and the
protocol name as `label`. They are driven by their own pollers, which need
hardware to develop against.

Bluetooth is deliberately absent. While the phone is connected over BLE the
Flipper's radio is serving that link and cannot scan for other devices, so the
app reports the chip as unavailable rather than leaving the user to read silence
as "nothing nearby".

## Firmware targets

Built against the official SDK. Custom firmwares (Momentum, Unleashed,
RogueMaster) expose extra frequencies and protocols; the app selects a `.fap`
built for the `api_version` it reads from `System.DeviceInfo`, so each firmware
family needs its own build of this directory.
