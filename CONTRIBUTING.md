# Contributing to FlipperZero Tool

Thank you for your interest in contributing! This document explains how to set up
the project for development.

## Prerequisites

- **Rust** (1.80+) — install via [rustup](https://rustup.rs)
- **Node.js** (20+) + npm
- **Tauri CLI** — `cargo install tauri-cli`
- ** Target platform deps:**
  - Windows: Visual Studio Build Tools with "Desktop development with C++"
  - Linux: `libwebkit2gtk-4.0-dev`, `libssl-dev`, `libgtk-3-dev`
  - macOS: Xcode Command Line Tools

## Setup

```bash
# Clone the repo
git clone https://github.com/giosue/flipperzero-tool.git
cd flipperzero-tool

# Install frontend dependencies
cd frontend
npm install

# Run in dev mode (hot reload)
cd ..
cargo tauri dev

# Build for production
cargo tauri build
```

## Project Structure

```
flipperzero-tool/
├── src-tauri/          # Rust backend
│   ├── src/
│   │   ├── main.rs     # Entry point
│   │   ├── lib.rs      # Module exports
│   │   ├── commands.rs # Tauri commands (FS operations)
│   │   ├── errors.rs   # AppError enum
│   │   ├── serial.rs   # Serial port communication
│   │   ├── parsers.rs  # File format parsers (sub/ir/nfc)
│   │   └── vfs.rs      # SQLite virtual filesystem cache
│   ├── Cargo.toml
│   └── tauri.conf.json
├── frontend/           # React frontend
│   ├── src/
│   │   ├── App.tsx     # Main application component
│   │   ├── main.tsx    # React entry point
│   │   └── services/
│   │       └── tauri.ts # Tauri API wrapper
│   └── package.json
├── .flipper_mock/       # Mock SD card for local development
└── .github/            # CI/CD workflows
```

## Code Style

### Rust
- Run `cargo clippy -- -D warnings` before submitting — zero warnings policy
- Run `cargo fmt` for consistent formatting
- No `unwrap()` in production code — use `AppError` variants
- Every public function must document errors via `Result<T, AppError>`

### TypeScript/React
- Run `tsc --noEmit` for type checking
- Run `eslint src/` for linting
- Components: PascalCase file names (e.g. `FileTable.tsx`)
- Hooks: camelCase, `use` prefix (e.g. `useDirectory`)
- No `any` types — use proper generics

## Commit Convention

```
feat: add serial port auto-detection
fix: resolve parser error on empty .sub files
docs: update README with new screenshots
refactor: extract FileTable component from App.tsx
test: add unit tests for commands.rs
```

## Pull Request Process

1. Fork the repository and create a feature branch
2. Make your changes, ensure CI passes (`cargo clippy`, `tsc --noEmit`)
3. Add tests for new functionality
4. Update `CHANGELOG.md` under `[Unreleased]`
5. Submit a PR with a clear description of the changes

## License

By contributing, you agree that your contributions will be licensed under the MIT License.
