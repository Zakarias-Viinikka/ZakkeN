#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ARTIFACTS_DIR="$SCRIPT_DIR/artifacts"
RUN_TS="$(date +%Y%m%d_%H%M%S)"
RUN_DIR="$ARTIFACTS_DIR/$RUN_TS"
ENV_FILE="$SCRIPT_DIR/ip.env"
mkdir -p "$RUN_DIR/screenshots"

APP_PACKAGE="${APP_PACKAGE:-z.zndroid}"
APP_ACTIVITY="${APP_ACTIVITY:-z.zndroid.MainActivity}"
APPIUM_URL="${APPIUM_URL:-http://127.0.0.1:4723}"

log() { echo "[run-ui-tests] $*" >&2; }

# AI shells don't source nvm, so `appium` (npm -g under nvm) isn't on PATH.
if [ -d "$HOME/.config/nvm/versions/node" ]; then
  for _nb in "$HOME/.config/nvm/versions/node"/*/bin; do
    [ -d "$_nb" ] || continue
    case ":$PATH:" in *":$_nb:"*) ;; *) export PATH="$_nb:$PATH" ;; esac
  done
fi

if [ -t 1 ]; then
  C_RESET=$'\033[0m'; C_DIM=$'\033[2m'; C_GREEN=$'\033[32m'; C_RED=$'\033[31m'; C_BOLD=$'\033[1m'
else
  C_RESET=""; C_DIM=""; C_GREEN=""; C_RED=""; C_BOLD=""
fi

if [ -z "${ANDROID_HOME:-}" ] && [ -z "${ANDROID_SDK_ROOT:-}" ]; then
  for candidate in "$HOME/Android/Sdk" "$HOME/Library/Android/sdk" "/usr/lib/android-sdk" "$HOME/Android/sdk"; do
    if [ -d "$candidate" ]; then
      export ANDROID_HOME="$candidate"; export ANDROID_SDK_ROOT="$candidate"
      log "ANDROID_HOME not set; using $candidate"; break
    fi
  done
fi
if [ -z "${ANDROID_HOME:-}" ] && [ -z "${ANDROID_SDK_ROOT:-}" ]; then
  log "Android SDK not found. Set ANDROID_HOME and re-run."; exit 7
fi

command -v adb >/dev/null 2>&1 || { log "adb not on PATH"; exit 2; }

dev_ready() { adb devices 2>/dev/null | awk 'NR>1 && $2=="device" && $1 !~ /^emulator-/ {found=1; exit} END{exit !found}'; }
try_connect() {
  local a="$1"; [ -z "$a" ] && return 1
  timeout 3 adb connect "$a" >/dev/null 2>&1 || true
  for _ in $(seq 1 10); do dev_ready && return 0; sleep 0.3; done
  return 1
}

if [ -f "$ENV_FILE" ]; then
  # shellcheck disable=SC1090
  source "$ENV_FILE"
fi
if [ -n "${PHONE_IP:-}" ] && [ -n "${PHONE_PORT:-}" ]; then
  PHONE_ADDR="$PHONE_IP:$PHONE_PORT"
else
  PHONE_ADDR=""
fi

if ! dev_ready; then
  adb kill-server 2>/dev/null || true
  sleep 0.5
  if [ -n "$PHONE_ADDR" ]; then
    log "connecting $PHONE_ADDR"
    try_connect "$PHONE_ADDR" || log "connect to $PHONE_ADDR didn't work"
  fi
fi

if ! dev_ready; then
  log "no device. run:  ./update-phone-ip.sh   (to set IP/port)   or   ./pair-device.sh (to re-pair)"
  adb devices >&2 || true
  exit 3
fi

SERIAL="$(adb devices 2>/dev/null | awk 'NR>1 && $2=="device" && $1 !~ /^emulator-/ {print $1; exit}')"
mapfile -t DEVICES < <(adb devices 2>/dev/null | awk 'NR>1 && $2=="device" {print $1}')
if [ "${#DEVICES[@]}" -ne 1 ]; then
  log "expected exactly one device, found ${#DEVICES[@]}"; adb devices >&2; exit 3
fi
[[ "$SERIAL" == emulator-* ]] && { log "refusing to run on emulator"; exit 4; }

if ! adb -s "$SERIAL" shell pm list packages 2>/dev/null | grep -qx "package:$APP_PACKAGE"; then
  log "app $APP_PACKAGE not installed on $SERIAL"; exit 5
fi

if ! curl -sf -m 2 "$APPIUM_URL/status" >/dev/null 2>&1; then
  command -v appium >/dev/null 2>&1 || { log "appium not installed"; exit 6; }
  log "starting appium (log: $RUN_DIR/appium.log)"
  appium --base-path / --port 4723 > "$RUN_DIR/appium.log" 2>&1 &
  APPIUM_PID=$!
  trap 'kill "$APPIUM_PID" 2>/dev/null || true' EXIT
  for _ in $(seq 1 120); do
    curl -sf -m 2 "$APPIUM_URL/status" >/dev/null 2>&1 && break
    sleep 0.5
  done
  if ! curl -sf -m 2 "$APPIUM_URL/status" >/dev/null 2>&1; then
    log "appium failed to start. Last lines:"; tail -20 "$RUN_DIR/appium.log" >&2 || true; exit 6
  fi
fi

if [ -d "$SCRIPT_DIR/.venv" ]; then source "$SCRIPT_DIR/.venv/bin/activate"; fi

export RUN_DIR APP_PACKAGE APP_ACTIVITY APPIUM_URL DEVICE_SERIAL="$SERIAL"

cd "$SCRIPT_DIR"
echo "${C_DIM}appium log: $RUN_DIR/appium.log${C_RESET}"
echo "${C_DIM}running test suite...${C_RESET}"

set +e
pytest tests/ --color=yes 2>&1 | tee "$RUN_DIR/pytest.log"
STATUS=${PIPESTATUS[0]}
set -e
sed -i 's/\033\[[0-9;]*m//g' "$RUN_DIR/pytest.log"

rm -rf "$ARTIFACTS_DIR/latest"
ln -s "$RUN_DIR" "$ARTIFACTS_DIR/latest" 2>/dev/null || \
  cp -r "$RUN_DIR" "$ARTIFACTS_DIR/latest" 2>/dev/null || true

if [ "$STATUS" -eq 0 ]; then _result=PASS; else _result=FAIL; fi
adb -s "$SERIAL" shell am broadcast \
  -a z.zndroid.debug.NOTIFY_TEST_DONE -p z.zndroid \
  --es result "$_result" --es summary "run $RUN_TS" >/dev/null 2>&1 || true

echo
if [ "$STATUS" -eq 0 ]; then
  echo "${C_BOLD}${C_GREEN}PASS${C_RESET}  ${C_DIM}$RUN_DIR${C_RESET}"
else
  echo "${C_BOLD}${C_RED}FAIL${C_RESET} (exit $STATUS)  ${C_DIM}$RUN_DIR${C_RESET}"
  echo "${C_RED}--- appium.log (last 20) ---${C_RESET}"
  tail -20 "$RUN_DIR/appium.log" 2>/dev/null | sed 's/^/  /'
  echo "${C_RED}--- end ---${C_RESET}"
fi
echo "${C_DIM}report:  $RUN_DIR/report.md${C_RESET}"
echo "${C_DIM}screens: $RUN_DIR/screenshots${C_RESET}"
echo "${C_DIM}logcat:  $RUN_DIR/logcat.txt${C_RESET}"

exit "$STATUS"