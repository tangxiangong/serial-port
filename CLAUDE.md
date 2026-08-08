# CLAUDE.md

## Project Overview

Serial Studio is a focused serial-port terminal for macOS. It uses Tauri 2
(Rust) and SolidJS (TypeScript). Serial communication is the only transport;
do not add BLE, networking, Modbus, or bridging without an explicit product
decision.

## Commands

```bash
bun install
bun run build
bun run tauri dev
cargo check --manifest-path src-tauri/Cargo.toml
cargo test --manifest-path src-tauri/Cargo.toml
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets
bun run format
```

Use Bun for frontend dependency and script commands.

## Architecture

- `src-tauri/src/serial/mod.rs`: single active serial session, split async
  reader/writer, frame encoding, statistics, and Tauri events.
- `src-tauri/src/serial/types.rs`: serial configuration and command/event DTOs.
- `src-tauri/src/serial/hotplug.rs`: lightweight port-list change watcher.
- `src-tauri/src/lib.rs`: five Tauri command boundaries only.
- `src/stores/serial.ts`: frontend orchestration and typed event subscriptions.
- `src/components/`: macOS-style connection, terminal, composer, toolbar, and
  status views.

Business rules and byte parsing stay in Rust. The frontend owns presentation,
preferences, and bounded in-memory terminal history.

Rust DTO fields use `#[serde(rename_all = "camelCase")]`; TypeScript interfaces
must match them exactly. Async read and write paths must not hold the same lock.
