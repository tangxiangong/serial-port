#!/usr/bin/env bash
set -euo pipefail
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
library="${1:?Pass the compiled serial-studio-ffi library}"
output="${2:-$root/bindings/generated/swift}"
cd "$root"
cargo run --locked -p serial-studio-bindgen -- generate "$library" --language swift --no-format --out-dir "$output"
