#!/usr/bin/env bash
set -euo pipefail
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
architecture="${ARCH:-$(uname -m)}"
case "$architecture" in arm64|x86_64) ;; *) echo "Unsupported architecture: $architecture" >&2; exit 2;; esac
case "${CONFIGURATION:-Debug}" in Debug|debug) configuration=Debug;; Release|release) configuration=Release;; *) echo "Unsupported configuration" >&2; exit 2;; esac
derived="${DERIVED_DATA_PATH:-$root/target/xcode/$architecture}"
args=(-project "$root/macos/SerialStudio.xcodeproj" -scheme SerialStudio -configuration "$configuration" -derivedDataPath "$derived" "ARCHS=$architecture" ONLY_ACTIVE_ARCH=YES "CODE_SIGN_IDENTITY=${CODE_SIGN_IDENTITY:--}")
if [[ -n "${APP_VERSION:-}" ]]; then args+=("MARKETING_VERSION=$APP_VERSION"); fi
if [[ -n "${BUILD_NUMBER:-}" ]]; then args+=("CURRENT_PROJECT_VERSION=$BUILD_NUMBER"); fi
xcodebuild "${args[@]}" build
source_app="$derived/Build/Products/$configuration/SerialStudio.app"
destination="$root/dist/macos/$architecture/SerialStudio.app"
mkdir -p "$(dirname "$destination")"
if [[ -d "$destination" ]]; then rm -rf "$destination"; fi
ditto "$source_app" "$destination"
if [[ -n "${SPARKLE_PUBLIC_ED_KEY:-}" ]]; then /usr/libexec/PlistBuddy -c "Add :SUPublicEDKey string $SPARKLE_PUBLIC_ED_KEY" "$destination/Contents/Info.plist"; fi
if [[ -n "${UPDATE_BASE_URL:-}" ]]; then /usr/libexec/PlistBuddy -c "Add :UpdateBaseURL string $UPDATE_BASE_URL" "$destination/Contents/Info.plist"; fi
# Keep Xcode's signing policy, including Debug entitlements and runtime flags.
# Forcing hardened runtime onto an ad hoc Debug launcher prevents it from loading
# SerialStudio.debug.dylib because neither binary has a signing team identifier.
codesign --force --sign "${CODE_SIGN_IDENTITY:--}" --preserve-metadata=entitlements,flags,runtime "$destination"
codesign --verify --deep --strict "$destination"
printf 'Built %s\n' "$destination"
