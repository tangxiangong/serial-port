# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Serial port debugging desktop application built with **Tauri 2** (Rust backend) + **SolidJS** (TypeScript frontend) + **DaisyUI/Tailwind CSS**. Supports serial (tokio-serial) and BLE (btleplug) communication, Modbus RTU/ASCII parsing, file logging, and serial port bridging.

## Build & Development Commands

```bash
# Package manager: bun (not npm/npx)
bun install                          # Install frontend dependencies

# Frontend only
bun run dev                          # Dev server on localhost:1420
bun run build                        # Production build to dist/

# Rust backend only
cargo check --manifest-path src-tauri/Cargo.toml
cargo test --manifest-path src-tauri/Cargo.toml
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets

# Run a single Rust test
cargo test --manifest-path src-tauri/Cargo.toml test_name

# Full Tauri app (builds both frontend + backend)
bun run tauri dev                    # Development with HMR
bun run tauri build                  # Production bundle

# Formatting
bun run format                       # Prettier for TS/TSX/CSS
```

No frontend test framework is configured. Backend tests use `#[cfg(test)]` modules (80 tests across codec, checksum, send, record, modbus, ring_buffer).

## Architecture

**All business logic lives in Rust.** The frontend is a pure UI layer — it calls Tauri commands and listens to events, but performs no data processing.

### Backend → Frontend Communication

Two patterns:

1. **Commands** (request-response): Frontend calls `invoke("command_name", { params })`, Rust handler returns `Result<T, String>`. Defined in `src-tauri/src/lib.rs`, wrapped in `src/lib/commands.ts`.

2. **Events** (push): Rust emits via `app_handle.emit("event:name", payload)`, frontend subscribes via `listen()`. Event types defined in `src/lib/events.ts`.

### Key Rust Modules

- **`lib.rs`** — Tauri command handlers (23 commands), AppState initialization, plugin registration
- **`state.rs`** — `AppState` struct holding all managers behind `Arc<Mutex<>>`
- **`serial/mod.rs`** — `SerialManager`: open/close/write ports, spawns background read tasks with 5ms frame aggregation via `RingBuffer`
- **`serial/hotplug.rs`** — Polls for USB device changes every 500ms, emits `serial:port_added/removed`
- **`ble/mod.rs`** — `BleManager`: scan/connect/write/disconnect via btleplug
- **`send.rs`** — `build_send_buffer()`: encodes input (hex/ascii/dec) + line ending + optional checksum
- **`codec.rs`** — Encoding/decoding between hex, ASCII, decimal byte representations
- **`checksum.rs`** — `CheckAlgorithm` enum, `apply_check()` for sum/parity/xor/modbus_crc16
- **`record.rs`** — `RecordManager` per port, tracks Rx/Tx byte counts
- **`modbus.rs`** — `parse_rtu_frame()` / `parse_ascii_frame()` with CRC/LRC validation
- **`bridge.rs`** — Bidirectional forwarding between two open serial ports
- **`file_logger.rs`** — `LogFormat` enum (Raw/Hex/Csv), writes to file with timestamps

### Frontend Structure

- **`src/lib/commands.ts`** — Type-safe `invoke()` wrappers for every Tauri command. Types here must match Rust serde serialization exactly.
- **`src/lib/events.ts`** — Event listener helpers with typed payloads
- **`src/stores/ui.ts`** — Global signals for panel visibility, theme, connection state
- **`src/stores/persistent.ts`** — Tauri Store plugin wrapper for settings persistence
- **`src/components/`** — UI components (all use DaisyUI classes)

## Type Bridging Convention

Rust enums use `#[serde(rename_all = "lowercase")]` or `"snake_case"` and structs use `#[serde(rename_all = "camelCase")]`. TypeScript types must match these serialized forms:

```rust
// Rust: snake_case fields → camelCase JSON
#[serde(rename_all = "camelCase")]
pub struct SerialConfig { pub baud_rate: u32, ... }

// Rust: enum variants → lowercase strings
#[serde(rename_all = "lowercase")]
pub enum Parity { None, Odd, Even }
```

```typescript
// TypeScript: matches JSON output
interface SerialConfig { baudRate: number; ... }
type Parity = "none" | "odd" | "even";
```

When adding a new Tauri command: define the Rust handler in `lib.rs`, register it in `generate_handler![]`, add the typed wrapper in `commands.ts`, and if it emits events, add the listener in `events.ts`.

## Rust Edition & Style

- Edition 2024. Uses `anyhow::Result` internally, converts to `String` errors at the Tauri command boundary via `.map_err(|e| e.to_string())`.
- Prefer typed enums over raw strings for command parameters.
- All async serial/BLE I/O uses tokio. Never block the async runtime.
