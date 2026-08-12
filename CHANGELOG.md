# Changelog

All notable changes to FlipperZero Tool will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Mobile port, phase P0 (foundations). `serialport` is now a desktop-only dependency, declared under
  a `cfg(not(android/ios))` target table, and every call site of it sits behind `#[cfg(desktop)]`
  with a mobile counterpart that fails with a clear "use the BLE transport" message instead of
  failing to compile. `run()` gained `#[cfg_attr(mobile, tauri::mobile_entry_point)]`
- `serial::base64_decode`, the missing counterpart to `base64_encode`. Without it binary payloads
  could be sent to the device but never read back, which blocks `.fap` deployment from a phone
- New `android-check` CI job: type-checks the crate for `aarch64-linux-android` via `cargo-ndk` and
  asserts `serialport` stays out of the Android dependency tree. The NDK is not available in every
  development environment, so this guard lives in CI. It is green, which settles an open question
  from the port plan: `rusqlite`'s bundled SQLite does cross-compile against the NDK
- Mobile port, phase P1 (transport and RPC). `transport::Transport` is the seam every link shares:
  USB CDC, BLE and Android USB-OTG move bytes differently but carry the same protobuf stream.
  `transport::usb_cdc` implements it for desktop and holds the port open for the transport's whole
  life, including the `start_rpc_session` handshake that switches the device out of CLI mode.
  `transport::loopback` is an in-memory link that reproduces BLE's small MTU and partial reads
- `rpc::framing` implements protobuf length-delimited framing, tolerating frames split across
  arbitrary read boundaries -- including a varint length prefix straddling two BLE notifications
- `rpc::FlipperSession` owns a connection for its lifetime and allocates RPC sequence ids
- Mobile port, phase P5 (Signal Radar). `signals::model` normalises every radio into one `Signal`,
  identified by a fingerprint over the fields that actually identify it -- excluding RSSI, time and
  location, which describe a sighting rather than the signal, so repeat encounters increment a count
  instead of piling up duplicates. `signals::store` keeps `signals` and `signal_sightings` as
  separate tables so the timeline, RSSI history and geotagging survive
- Signal Radar UI: `FlipperSchematic` (chip hotspots as real keyboard-reachable buttons, laid out
  where the radios physically sit), `ChipSheet`, `SignalCard` and `TransmitGate`
- Bilingual IT/EN interface via a flat key catalogue with interpolation and plural forms. The
  English catalogue is typed against the Italian one, so a key added to one and missing from the
  other fails the build rather than showing a raw key to a user
- Mobile port, phase P7: `.rfid` (125 kHz badges) and `.ibtn` (iButton contact keys) parsers, both
  loose and typed, with Tauri commands registered. These were the only two Flipper formats the app
  could not read at all

### Changed
- `run()` no longer calls `std::process::exit` on a fatal error; it panics instead. Self-terminating
  is not a legal way to leave a mobile app and iOS reports it as a crash
- `docs/MOBILE-PORT-PLAN.md`: architecture and 8-phase delivery plan for the iOS/Android port and the
  new Signal Radar view, plus `docs/AGENT-PROMPT-MOBILE-PORT.md` with the implementation prompt

### Security
- Frontend: resolved 4 high-severity advisories in transitive dependencies via `npm audit fix`
  (`nanoid` 3.3.12 → 3.3.18, `postcss` 8.5.15 → 8.5.26, `undici` 7.28.0 → 7.29.0,
  `brace-expansion` 5.0.6 → 5.0.9). Lockfile only — `package.json` unchanged, all bumps
  semver-compatible. Unblocks the `security-scan` CI job, which was failing on every branch

### Fixed
- `proto_bus::rpc_command` was a stub that always returned "ProtoBus serial integration pending",
  leaving the tested protobuf codec with nothing underneath it and every `proto_*` helper dead. It
  now runs over a `FlipperSession`, matching replies by `sequence_id` so unsolicited device messages
  (and Signal Radar FAP telemetry) cannot be mistaken for a response and desynchronise the stream
- `serial_upload` rejected any file that was not valid UTF-8, and `serial_download` forced downloads
  through a UTF-8 round-trip. Both now move bytes, so binary files on the SD card survive the trip
- Six commands defined in `commands.rs` were never registered in the `invoke_handler` list
  (`parser_parse_{sub,ir,nfc}_struct`, `template_{get,list,create}`). The Sub-GHz/IR/NFC detail views
  already invoked them, so those calls failed at runtime
- parsers.rs: changed return type from `Result<ParsedFile, String>` to `Result<ParsedFile, AppError>` for consistency
- App.tsx: removed hardcoded Windows path `MOCK_ROOT`, now uses localStorage for root directory
- Backend no longer fails to compile: resolved the `serial.rs`/`serial/mod.rs` module conflict,
  gave `main.rs` a working module tree (now delegates to `flipperzero_tool_lib::run()`), fixed an
  unclosed `impl` block in `parsers.rs`, and removed a `proto_bus.rs` import of two functions that
  never existed
- `reverse_engineer.rs`: pattern-match sort used `.partial_cmp().unwrap()`, which panics on NaN
  confidence values; switched to `total_cmp`
- Two pre-existing test bugs: a create-file "already exists" test that wrote to the wrong filename,
  and `parse_sub` unconditionally emitting `filetype`/`version` fields even for empty input
- `proto_bus.rs`: `decode_file_list` broke out of its field-scanning loop on the very first field
  (`name`) instead of skipping past it, so `size`/`is_dir` were silently never decoded
- Frontend: removed all remaining `any` types (parser result handlers in the Sub-GHz/IR/NFC viewers,
  the CodeMirror keymap cast, and `tauri.ts`'s parser wrappers), replaced with real types matching
  the Rust structs. Along the way, fixed `SubGhzViewer` referencing `data.frequency_display`, a
  field the backend never actually sent
- CI: `rust-lint`/`rust-test` were missing `libudev-dev` on the Linux runner, which `serialport`
  needs to link against — both jobs had been failing on every push
- Resolved 85 pre-existing `eslint-plugin-react-hooks` v7 findings that predated this branch and had
  been blocking `frontend-lint` (and therefore CI/auto-merge) since before this work started: several
  real bugs where a ref was mutated or `setState` called synchronously during render (CodeMirror's
  `onChangeRef`, `useEditor`'s view/mock-mode refs, a `saveFile`-before-declared ordering issue,
  `UfbtPanel`'s `checkUfbt` ordering), 27 re-thrown errors in `tauri.ts` missing an attached `cause`,
  `Toast.tsx` mixing a component export with non-component exports (split into `lib/toastStore.ts`),
  and assorted dead code/unused imports

### Added
- `thiserror = "1"` added to Cargo.toml (was used implicitly)
- `@tauri-apps/api` added to frontend dependencies
- Tauri v2 plugin permissions configured in tauri.conf.json (fs scope + shell)
- LICENSE (MIT)
- CONTRIBUTING.md with development setup instructions
- Unit test coverage for `vfs.rs` (SQL logic split out from the `AppHandle`-dependent wrapper so it's
  testable against an in-memory SQLite connection) and `proto_bus.rs` (varint/RPC message encode-decode)
- `Cargo.lock` is now committed (previously gitignored) for reproducible application builds
- Frontend test infrastructure: vitest, jsdom, and Testing Library, wired up via `npm test`, with
  initial coverage for `tauri.ts`'s `AppError`-normalization logic
- CI: `frontend-test` job (runs the vitest suite) and `security-scan` job (`cargo audit` against
  RUSTSEC advisories, `npm audit --audit-level=high`)

### Removed
- Dead build-time dependency on `protoc`/`prost` codegen that nothing in `src/` ever consumed
- Unused Rust dependencies: `nom`, `notify`, `rfd`, `log` (the latter is still pulled in transitively
  via `env_logger`, which is the only thing that needs it)

## [0.1.0] - 2026-01-15

### Added
- Initial release
- Local file manager with mock SD card browser
- Serial port connection to Flipper Zero (basic)
- Key-value parser for .sub, .ir, .nfc files
- File editor with Local/Serial mode toggle
