# Changelog

All notable changes to FlipperZero Tool will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Fixed
- parsers.rs: changed return type from `Result<ParsedFile, String>` to `Result<ParsedFile, AppError>` for consistency
- App.tsx: removed hardcoded Windows path `MOCK_ROOT`, now uses localStorage for root directory

### Added
- `thiserror = "1"` added to Cargo.toml (was used implicitly)
- `@tauri-apps/api` added to frontend dependencies
- Tauri v2 plugin permissions configured in tauri.conf.json (fs scope + shell)
- LICENSE (MIT)
- CONTRIBUTING.md with development setup instructions

## [0.1.0] - 2026-01-15

### Added
- Initial release
- Local file manager with mock SD card browser
- Serial port connection to Flipper Zero (basic)
- Key-value parser for .sub, .ir, .nfc files
- File editor with Local/Serial mode toggle
