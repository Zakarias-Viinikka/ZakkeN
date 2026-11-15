#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

if [ -t 1 ]; then
  C_RESET=$'\033[0m'; C_DIM=$'\033[2m'; C_GREEN=$'\033[32m'; C_RED=$'\033[31m'
else
  C_RESET=""; C_DIM=""; C_GREEN=""; C_RED=""
fi

log()  { echo "${C_DIM}[build]${C_RESET} $*" >&2; }
fail() { echo "${C_RED}[build]${C_RESET} $*" >&2; exit 1; }
ok()   { echo "${C_GREEN}[build]${C_RESET} $*" >&2; }

APP_PACKAGE="${APP_PACKAGE:-z.zndroid}"

if [ -z "${ANDROID_HOME:-}" ] && [ -z "${ANDROID_SDK_ROOT:-}" ]; then
  for c in "$HOME/Android/Sdk" "$HOME/Library/Android/sdk" "/usr/lib/android-sdk"; do
    if [ -d "$c" ]; then
      export ANDROID_HOME="$c"; export ANDROID_SDK_ROOT="$c"
      log "ANDROID_HOME not set; using $c"
      break
    fi
  done
fi
[ -z "${ANDROID_HOME:-}" ] && [ -z "${ANDROID_SDK_ROOT:-}" ] && fail "Android SDK not found. Set ANDROID_HOME."
command -v adb >/dev/null 2>&1 || fail "adb not on PATH"

mapfile -t DEVICES < <(adb devices 2>/dev/null | awk 'NR>1 && $2=="device" {print $1}')
[ "${#DEVICES[@]}" -eq 0 ] && fail "no device connected. Run e2e_android_tests/update-phone-ip.sh or pair-device.sh first."

if [ -n "${ANDROID_SERIAL:-}" ]; then
  SERIAL=""
  for d in "${DEVICES[@]}"; do
    [ "$d" = "$ANDROID_SERIAL" ] || continue
    SERIAL="$ANDROID_SERIAL"; break
  done
  [ -z "$SERIAL" ] && fail "ANDROID_SERIAL=$ANDROID_SERIAL not connected (have: ${DEVICES[*]})"
else
  phys=()
  for d in "${DEVICES[@]}"; do
    [[ "$d" == emulator-* ]] || phys+=("$d")
  done
  [ "${#phys[@]}" -eq 0 ] && fail "only emulator(s) connected; refusing. Connect a physical phone."
  [ "${#phys[@]}" -gt 1 ] && fail "multiple physical devices; export ANDROID_SERIAL=<one of: ${phys[*]}>"
  SERIAL="${phys[0]}"
fi
export ANDROID_SERIAL="$SERIAL"
log "target: $SERIAL"

if [ -x ./gradlew ]; then GRADLE=./gradlew
elif command -v gradle >/dev/null 2>&1; then GRADLE=gradle
else fail "no gradlew and no gradle on PATH"
fi

log "schema drift check"
if [ -x ./schema_drift_detector/check.sh ]; then
  if ! ./schema_drift_detector/check.sh; then
    fail "schema drift detected — see the report above"
  fi
  export SCHEMA_DRIFT_CHECKED=1
else
  log "schema_drift_detector/check.sh missing; skipping"
fi

log "gradle :app:assembleDebug"
"$GRADLE" :app:assembleDebug

APK="$(find "$SCRIPT_DIR/app/build/outputs/apk/debug" -maxdepth 2 -type f -name '*.apk' 2>/dev/null | head -1)"
[ -z "$APK" ] && fail "no APK at app/build/outputs/apk/debug"
log "apk: $APK"

log "installing on $SERIAL..."
adb -s "$SERIAL" install -r "$APK"

# Capture first, grep second. Piping dumpsys directly into `grep -q`
# under `set -o pipefail` can report failure when adb is killed by
# SIGPIPE right after grep's early exit, producing a false "NOT
# registered" even when the receiver is there.
DUMP_PKG="$(adb -s "$SERIAL" shell dumpsys package "$APP_PACKAGE" 2>/dev/null || true)"
if printf '%s' "$DUMP_PKG" | grep -q "z.zndroid.debug.BEGIN_TEST_TX"; then
  ok "installed; TestTransactionReceiver is registered"
else
  echo "${C_RED}[build]${C_RESET} installed, but z.zndroid.debug.BEGIN_TEST_TX is NOT registered." >&2
  echo "${C_RED}[build]${C_RESET} The e2e transaction hook won't work. Check the receiver file is in src/debug/." >&2
  exit 4
fi
