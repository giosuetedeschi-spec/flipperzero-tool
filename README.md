# FlipperZero Tool

Desktop application for managing Flipper Zero files, reverse engineering, and plugin development.

## Tech Stack
- **Desktop Shell**: Tauri v2 (Rust backend, WebView frontend)
- **Frontend**: React + TypeScript + TailwindCSS + shadcn/ui
- **Backend**: Rust (serialport, prost/protobuf, rusqlite, nom)
- **Serial**: USB Virtual COM Port + Protocol Buffers

## Features (WIP)
- [x] Serial port listing and connection
- [x] File manager with caching (SQLite VFS)
- [x] Key-value parser for .sub, .ir, .nfc formats
- [x] uFBT integration for .fap plugin development
- [ ] Drag-and-drop file transfer
- [ ] Reverse engineering visualizer
- [ ] Built-in editor with syntax highlighting
- [ ] Protobuf communication layer

## Setup

### Prerequisites
- Rust 1.77+
- Node.js 20+
- npm

### Install
```bash
cd frontend && npm install
cd src-tauri && cargo build
```

### Dev
```bash
npx tauri dev
```

### Build
```bash
npx tauri build
```
