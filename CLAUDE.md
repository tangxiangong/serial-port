# Serial Studio

Native desktop serial utility. Follow AGENTS.md and the user's instructions.

## Architecture

- `crates/serial-studio-core`: serial transport, encoding and records.
- `crates/serial-studio-service`: desktop runtime and serialized operations.
- `crates/serial-studio-ffi`: UniFFI remote records and Swift exports.
- `crates/serial-studio-csharp`: Interoptopus Wire exports and inventory.
- `bindings/`: official generators; generated output is not source.
- `macos/SerialStudio.xcodeproj`: SwiftUI + Sparkle; RustBindings dependency.
- `windows/SerialStudio.sln`: WinUI + Velopack; MSBuild Rust targets.
- `xtask/`, `ci/`: platform announcements, signed feeds and native packaging.

## Checks

Start with targeted host-native logic/static checks. Do not add GUI automation tests.
Use cargo fmt, targeted cargo test and cargo clippy as appropriate.
Do not substitute macOS Windows/Linux cross-builds for native platform validation.
Hardware, signing/notarization, installation and update acceptance are separate checks.

## Development

Open the Xcode project or VS solution directly. Both generate bindings before compiling.
`script/build_and_run.sh` is the macOS build/run entrypoint.
`windows/scripts/build.ps1` uses the same project graph as Visual Studio.
Keep `macos/project.yml`, its generated project and shared scheme in sync.
Do not restore Tauri, frontend package tooling, handwritten ABI declarations or JSON bridge dispatch.
