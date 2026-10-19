#!/usr/bin/env bash
set -euo pipefail
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
for variable in SPARKLE_PRIVATE_ED_KEY SPARKLE_PUBLIC_ED_KEY APP_VERSION BUILD_NUMBER ARCH; do
  [[ -n "${!variable:-}" ]] || { echo "Missing release configuration: $variable" >&2; exit 1; }
done
work="$(mktemp -d "${TMPDIR:-/tmp}/serial-studio-release.XXXXXX")"
trap 'rm -rf "$work"' EXIT
umask 077
printf '%s' "$SPARKLE_PRIVATE_ED_KEY" > "$work/sparkle.key"
export CONFIGURATION=Release
bash "$root/macos/build.sh"
app="$root/dist/macos/$ARCH/SerialStudio.app"
output="$root/dist/release/macos-$ARCH"
if [[ -d "$output" && -n "$(ls -A "$output")" ]]; then echo "Release output must be empty: $output" >&2; exit 1; fi
mkdir -p "$output" "$work/archives" "$work/dmg"
archive="SerialStudio-$APP_VERSION-macos-$ARCH.app.zip"
ditto -c -k --keepParent "$app" "$work/archives/$archive"
sparkle="${DERIVED_DATA_PATH:-$root/target/xcode/$ARCH}/SourcePackages/artifacts/sparkle/Sparkle/bin"
"$sparkle/generate_appcast" --ed-key-file "$work/sparkle.key" --maximum-deltas 0 \
  --download-url-prefix "https://github.com/tangxiangong/serial-port/releases/download/v$APP_VERSION/" \
  -o "$output/appcast-$ARCH.xml" "$work/archives"
cp "$work/archives/$archive" "$output/$archive"
ditto "$app" "$work/dmg/SerialStudio.app"
ln -s /Applications "$work/dmg/Applications"
hdiutil create -quiet -volname 'Serial Studio' -srcfolder "$work/dmg" -format UDZO "$output/SerialStudio-$APP_VERSION-macos-$ARCH.dmg"
