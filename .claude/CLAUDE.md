# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project overview

Desktop application (Tauri v2) for managing Flipper Zero files, reverse engineering `.sub`/`.ir`/`.nfc` files, and developing `.fap` plugins via uFBT. Rust backend + React/TypeScript frontend, connected over Tauri's `invoke` bridge.

## Commands

### Frontend (run from `frontend/`)
```bash
npm install          # install deps
npm run dev           # vite dev server (port 5173)
npm run build          # tsc -b && vite build
npx tsc --noEmit        # type check only
npx eslint src/          # lint
```

### Backend (run from `src-tauri/`)
```bash
cargo check                    # fast compile check
cargo build
cargo test                     # unit + integration tests (src-tauri/tests/*.rs)
cargo test <test_name>          # run a single test
cargo clippy -- -D warnings      # zero-warnings lint policy, required before PRs
cargo fmt                        # formatting, also required before PRs
```

### Full app (run from repo root, requires Tauri CLI: `cargo install tauri-cli`)
```bash
npx tauri dev      # hot-reload desktop app (spawns frontend dev server automatically)
npx tauri build    # production bundle
```

### Python parser tests
There is a standalone Python port of the parsing logic at `src-tauri/python/parsers.py`, tested by `tests/test_parsers.py` (stdlib `unittest`, no pytest). Run with:
```bash
python tests/test_parsers.py
```
This is separate from the Rust parser implementation (`src-tauri/src/parsers.rs`) — keep both in sync if you change key-value parsing behavior for `.sub`/`.ir`/`.nfc` formats.

## Architecture

`src-tauri/src/main.rs` is a thin entry point (`flipperzero_tool_lib::run()`); `src-tauri/src/lib.rs` declares all 8 modules (`errors`, `commands`, `serial`, `ufbt`, `vfs`, `parsers`, `reverse_engineer`, `proto_bus`) and re-exports their public API for both the binary and the `flipperzero_tool_lib` crate used by `src-tauri/tests/*.rs`.

### Backend module responsibilities (`src-tauri/src/`)
- `commands.rs` — all `#[tauri::command]` entry points (local FS CRUD, serial ops, VFS cache ops, parser invocation, uFBT project ops). This is the surface the frontend calls via `invoke()`.
- `errors.rs` — single `AppError` enum (`thiserror`-derived, `Serialize`/`Deserialize`) used as the `Err` type everywhere; converts from `std::io::Error` and `rusqlite::Error`. No `unwrap()` in production code — return `AppError` variants instead.
- `serial/mod.rs` — USB VID:PID auto-detect (Flipper VID `0x0483`, PID `0x5740`), CLI-based file operations over the serial port, varint framing, connection state via `Arc<Mutex<...>>`.
- `proto_bus.rs` — hand-rolled Protobuf wire-format encode/decode (varint/LEB128, length-prefixed message framing) for the Flipper RPC protocol defined in `src-tauri/proto/flipper.proto`. Session/sequence-ID tracking. Full serial wiring is still WIP per `docs/FEATURE-PROTOBUS.md` — encode/decode work offline but `FlipperConnection` doesn't yet hold a port handle to actually transmit RPC frames.
- `vfs.rs` — SQLite-backed cache (`rusqlite`) of the Flipper filesystem tree + file contents, stored under the Tauri app data dir (`flipper_cache.db`), so the frontend can browse without re-querying the device every time.
- `parsers.rs` — key-value parser for Flipper's `.sub`/`.ir`/`.nfc` file formats (`nom`-based). Mirrored in Python at `src-tauri/python/parsers.py` for the test suite in `tests/`.
- `reverse_engineer.rs` — analysis logic backing the "Reverse Engineer" panel in the frontend.
- `ufbt.rs` — wraps the `ufbt` CLI (Flipper's micro build tool) for `.fap` plugin project scaffolding, build, deploy.

### Frontend structure (`frontend/src/`)
- `services/tauri.ts` — the only place that calls `invoke()`. Every backend command has a typed wrapper here that normalizes `AppError` (an object with one of several optional keys) into a plain `Error`. Add new wrappers here rather than calling `invoke()` directly from components.
- `App.tsx` — top-level state machine for `viewMode` (`"local" | "serial"`), owns the mock-mode toggle, and composes the custom hooks below.
- `hooks/useDirectory.ts`, `useEditor.ts`, `useDragDrop.ts`, `useFileActions.ts` — encapsulate directory listing, multi-tab editor state (dirty tracking, find/replace, autosave), and drag-and-drop file moves respectively.
- `components/editor/CodeMirrorEditor.tsx` — CodeMirror 6 integration for the built-in editor.
- Viewer components (`SubGhzViewer(Advanced)`, `NfcViewer`, `NfcAnalyzerAdvanced`, `IrViewer`, `IrDatabase`) render the parsed output of `parsers.rs`/`parsers.py` for their respective file types.
- `DevicePanel.tsx` has a "mock Flipper" mode (see `App.tsx`'s `mockMode` state) that simulates a connected device using `.flipper_mock/` at the repo root instead of a real serial connection — useful for UI development without hardware.

### `.flipper_mock/`
A fake SD card layout (`badusb/`, `ir/`, `nfc/`, `subghz/`) used as the mock device root during local development — not real device data.

## Conventions (from CONTRIBUTING.md)

- Rust: `cargo clippy -- -D warnings` and `cargo fmt` must be clean before submitting; no `unwrap()` in production code; every public function returns `Result<T, AppError>`.
- TypeScript: `tsc --noEmit` and `eslint src/` must pass; no `any` types; components are PascalCase files, hooks are camelCase with a `use` prefix.
- Commit messages follow Conventional Commits (`feat:`, `fix:`, `docs:`, `refactor:`, `test:`).
- Update `CHANGELOG.md` under `[Unreleased]` for notable changes.
- `docs/FEATURE-*.md` are Italian-language implementation notes written per feature (ProtoBus, Editor, Drag&Drop, Reverse Engineer) — check the relevant one before extending that feature, they document known gaps (e.g. ProtoBus's incomplete serial wiring above).

## Known issues to be aware of

- `scripts/` contains personal remote-deploy scripts (`deploy.py`, `deploy2.py`, `_deploy_and_test.py`, etc.) that SSH into a hardcoded LAN host with hardcoded weak credentials. These are developer scratch tooling, not part of the app or CI — don't invoke them, and don't use them as a model for new tooling.
