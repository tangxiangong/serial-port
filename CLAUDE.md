# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

A desktop serial port / BLE debugging assistant built with Rust. Uses **GPUI** (via `gpui` + `gpui-component`) for the native GUI and **Tokio** for async I/O. The UI is in Chinese (zh-CN).

## Build & Run

```bash
cargo build          # debug build
cargo run            # run the app
cargo build --release  # release build
```

No test suite exists yet. No linter configuration beyond standard `cargo clippy`.

## Architecture

The codebase follows a layered architecture with three main layers:

- **`core/`** — Pure domain logic with no framework dependencies: codecs (hex/text encoding via `encoding_rs`), checksum algorithms, record types (`RecordEntry`, `SessionMeta`, `QuickInputItem`), and full-text search.
- **`infra/`** — Infrastructure adapters:
  - `transport/` — `AsyncTransport` trait with two implementations: `serial` (via `tokio-serial`) and `ble` (via `btleplug`). Both produce a `TransportEvent` stream.
  - `storage.rs` — `SessionStore` for persisting sessions/settings to disk (under `.serial-port-data/`).
- **`app/`** — Application orchestration:
  - `controller.rs` — `AppController` owns a Tokio runtime handle, processes `AppCommand` variants, and emits `AppEvent` variants over an mpsc channel. All async work (connect, send, auto-send, search) is spawned here.
  - `search_service.rs` — Async search across stored sessions.
- **`state.rs`** — All UI state in `AppState` plus configuration types (`AppSettings`, `SerialConfig`, `BleProfile`).
- **`view.rs`** — Single `AppView` struct implementing GPUI's `Render` trait. Subscribes to `AppEvent` and dispatches `AppCommand`.
- **`main.rs`** — Bootstrap: builds a Tokio multi-thread runtime, launches the GPUI application, wires controller to view.

### Key data flow

`AppView` → `AppCommand` → `AppController` (spawns async on Tokio) → `AppEvent` → `AppView` updates `AppState`.

Transport connections produce `TransportEvent` streams that the controller forwards as `AppEvent::RecordCommitted` / `LiveReadChunk`.

## Notable Dependencies

- `gpui` / `gpui-component` — Zed's GPU-accelerated UI framework (macOS-only currently)
- `tokio-serial` — Async serial port I/O
- `btleplug` — Cross-platform BLE
- `encoding_rs` — Character encoding (GBK, etc.)

## Data Storage

App data (settings, sessions) is persisted under `.serial-port-data/` in the project root (gitignored).
