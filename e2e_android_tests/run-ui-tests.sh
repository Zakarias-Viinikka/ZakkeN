#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ARTIFACTS_DIR="$SCRIPT_DIR/artifacts"
RUN_TS="$(date +%Y%m%d_%H%M%S)"
RUN_DIR="$ARTIFACTS_DIR/$RUN_TS"
mkdir -p "$RUN_DIR/screenshots"

APP_PACKAGE="${APP_PACKAGE:-z.zndroid}"
APP_ACTIVITY="${APP_ACTIVITY:-z.zndroid.MainActivity}"
APPIUM_URL="${APPIUM_URL:-http://127.0.0.1:4723}"

log() { echo "[run-ui-tests] $*" >&2; }

if [ -t 1 ]; then
  C_RESET=$'\033[0m'
  C_DIM=$'\033[2m'
  C_GREEN=$'\033[32m'
  C_RED=$'\033[31m'
  C_BOLD=$'\033[1m'
else
  C_RESET=""; C_DIM=""; C_GREEN=""; C_RED=""; C_BOLD=""
fi

if [ -z "${ANDROID_HOME:-}" ] && [ -z "${ANDROID_SDK_ROOT:-}" ]; then
  for candidate in "$HOME/Android/Sdk" "$HOME/Library/Android/sdk" "/usr/lib/android-sdk" "$HOME/Android/sdk"; do
    if [ -d "$candidate" ]; then
      export ANDROID_HOME="$candidate"
      export ANDROID_SDK_ROOT="$candidate"
      log "ANDROID_HOME not set; using $candidate"
      break
    fi
  done
fi
if [ -z "${ANDROID_HOME:-}" ] && [ -z "${ANDROID_SDK_ROOT:-}" ]; then
  log "Android SDK not found. Set ANDROID_HOME (e.g. export ANDROID_HOME=\$HOME/Android/Sdk) and re-run."
  exit 7
fi

if ! command -v adb >/dev/null 2>&1; then
  log "adb not found in PATH. Install Android platform-tools and add to PATH."
  exit 2
fi

mapfile -t DEVICES < <(adb devices | awk 'NR>1 && $2=="device" {print $1}')
if [ "${#DEVICES[@]}" -ne 1 ]; then
  log "Expected exactly one connected device, found ${#DEVICES[@]}."
  adb devices >&2 || true
  exit 3
fi
SERIAL="${DEVICES[0]}"

if [[ "$SERIAL" == emulator-* ]]; then
  log "Device '$SERIAL' looks like an emulator. Refusing to run on emulator."
  exit 4
fi

if ! adb -s "$SERIAL" shell pm list packages 2>/dev/null | grep -qx "package:$APP_PACKAGE"; then
  log "App '$APP_PACKAGE' not installed on '$SERIAL'."
  exit 5
fi

if ! curl -sf -m 2 "$APPIUM_URL/status" >/dev/null 2>&1; then
  if ! command -v appium >/dev/null 2>&1; then
    log "Appium not installed. Run: npm i -g appium && appium driver install uiautomator2"
    exit 6
  fi
  log "Appium not running; starting on port 4723 (log: $RUN_DIR/appium.log)"
  appium --base-path / --port 4723 > "$RUN_DIR/appium.log" 2>&1 &
  APPIUM_PID=$!
  trap 'kill "$APPIUM_PID" 2>/dev/null || true' EXIT
  for _ in $(seq 1 120); do
    curl -sf -m 2 "$APPIUM_URL/status" >/dev/null 2>&1 && break
    sleep 0.5
  done
  if ! curl -sf -m 2 "$APPIUM_URL/status" >/dev/null 2>&1; then
    log "Appium failed to start. Last lines of $RUN_DIR/appium.log:"
    tail -20 "$RUN_DIR/appium.log" >&2 || true
    exit 6
  fi
fi

if [ -d "$SCRIPT_DIR/.venv" ]; then
  # shellcheck disable=SC1091
  source "$SCRIPT_DIR/.venv/bin/activate"
fi

export RUN_DIR APP_PACKAGE APP_ACTIVITY APPIUM_URL DEVICE_SERIAL="$SERIAL"

cd "$SCRIPT_DIR"
echo "${C_DIM}appium log: $RUN_DIR/appium.log  (full detail; set E2E_TAIL_APPIUM=0 to silence the filtered stream)${C_RESET}"
echo "${C_DIM}running test suite (first run installs helper APKs on the phone, 1-2 min over wireless)...${C_RESET}"

TAIL_PID=""
if [ "${E2E_TAIL_APPIUM:-1}" != "0" ] && [ -f "$RUN_DIR/appium.log" ]; then
  ( tail -n 0 -F "$RUN_DIR/appium.log" 2>/dev/null \
    | grep --line-buffered -E "Installing|Uninstalling|Starting session|Creating session|Created session|Removing" \
    | sed 's/^/  [appium] /' >&2 ) &
  TAIL_PID=$!
fi

set +e
pytest tests/ --color=yes 2>&1 | tee "$RUN_DIR/pytest.log"
STATUS=${PIPESTATUS[0]}
set -e
if [ -n "$TAIL_PID" ]; then kill "$TAIL_PID" 2>/dev/null || true; fi
sed -i 's/\033\[[0-9;]*m//g' "$RUN_DIR/pytest.log"

rm -rf "$ARTIFACTS_DIR/latest"
ln -s "$RUN_DIR" "$ARTIFACTS_DIR/latest" 2>/dev/null || \
  cp -r "$RUN_DIR" "$ARTIFACTS_DIR/latest" 2>/dev/null || true

echo
if [ "$STATUS" -eq 0 ]; then
  echo "${C_BOLD}${C_GREEN}PASS${C_RESET}  ${C_DIM}$RUN_DIR${C_RESET}"
else
  echo "${C_BOLD}${C_RED}FAIL${C_RESET} (exit $STATUS)  ${C_DIM}$RUN_DIR${C_RESET}"
fi
echo "${C_DIM}report:  $RUN_DIR/report.md${C_RESET}"
echo "${C_DIM}screens: $RUN_DIR/screenshots${C_RESET}"
echo "${C_DIM}logcat:  $RUN_DIR/logcat.txt${C_RESET}"

exit "$STATUS"
