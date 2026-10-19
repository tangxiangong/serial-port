#!/usr/bin/env bash
# Xcode's RustBindings target owns Cargo and official UniFFI generation.
set -euo pipefail
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
export PATH="${HOME}/.cargo/bin:$PATH"
cd "$root"
case "${CONFIGURATION:-Debug}" in Debug|debug) profile=debug;; Release|release) profile=release;; *) echo "Unsupported configuration" >&2; exit 2;; esac
output="$root/bindings/generated/swift"
architectures="${ARCHS:-$(uname -m)}"
if [[ "${ONLY_ACTIVE_ARCH:-NO}" == YES && -n "${NATIVE_ARCH_ACTUAL:-}" ]]; then
  case " $architectures " in *" $NATIVE_ARCH_ACTUAL "*) architectures="$NATIVE_ARCH_ACTUAL";; esac
fi
mkdir -p "$output/include" "$output/raw"
archives=()
metadata=""
for architecture in $architectures; do
  case "$architecture" in arm64) target=aarch64-apple-darwin;; x86_64) target=x86_64-apple-darwin;; *) echo "Unsupported architecture: $architecture" >&2; exit 2;; esac
  if command -v rustup >/dev/null 2>&1 && ! rustup target list --installed | grep -Fxq "$target"; then
    rustup target add "$target"
  fi
  # Bash 3.2 treats an empty array expansion as unset under `set -u`.
  # Keep the argument array nonempty in both Debug and Release builds.
  cargo_args=(build --locked --target-dir "$root/target" -p serial-studio-ffi --target "$target")
  if [[ "$profile" == release ]]; then cargo_args+=(--release); fi
  cargo "${cargo_args[@]}"
  archives+=("$root/target/$target/$profile/libserial_studio_ffi.a")
  metadata="$root/target/$target/$profile/libserial_studio_ffi.dylib"
done
bash "$root/bindings/generate-swift.sh" "$metadata" "$output/raw"
copy_changed() { if ! cmp -s "$1" "$2"; then cp "$1" "$2"; fi; }
copy_changed "$output/raw/SerialStudioCore.swift" "$output/SerialStudioCore.swift"
copy_changed "$output/raw/SerialStudioCoreFFI.h" "$output/include/SerialStudioCoreFFI.h"
copy_changed "$output/raw/SerialStudioCoreFFI.modulemap" "$output/include/module.modulemap"
if [[ ${#archives[@]} -eq 1 ]]; then copy_changed "${archives[0]}" "$output/libserial_studio_ffi.a";
else xcrun lipo -create "${archives[@]}" -output "$output/libserial_studio_ffi.a"; fi
