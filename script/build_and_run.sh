#!/usr/bin/env bash
set -euo pipefail
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
mode="${1:-run}"
case "$mode" in run|--debug|--logs|--telemetry|--verify) ;; *) echo "usage: $0 [run|--debug|--logs|--telemetry|--verify]" >&2; exit 2;; esac
pkill -x SerialStudio >/dev/null 2>&1 || true
bash "$root/macos/build.sh"
app="$root/dist/macos/${ARCH:-$(uname -m)}/SerialStudio.app"
case "$mode" in
  run) /usr/bin/open -n "$app";;
  --debug) lldb -- "$app/Contents/MacOS/SerialStudio";;
  --logs|--telemetry) /usr/bin/open -n "$app"; /usr/bin/log stream --info --style compact --predicate 'process == "SerialStudio"';;
  --verify) /usr/bin/open -n "$app"; sleep 1; pgrep -x SerialStudio >/dev/null;;
esac
