#!/usr/bin/env bash
set -uo pipefail
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ENV_FILE="$SCRIPT_DIR/ip.env"

log() { echo "[pair-device] $*" >&2; }
ready() { adb devices 2>/dev/null | awk 'NR>1 && $2=="device" && $1 !~ /^emulator-/ {found=1; exit} END{exit !found}'; }
serial() { adb devices 2>/dev/null | awk 'NR>1 && $2=="device" && $1 !~ /^emulator-/ {print $1; exit}'; }
try_connect() {
  local a="$1"; [ -z "$a" ] && return 1
  timeout 3 adb connect "$a" >/dev/null 2>&1 || true
  for _ in $(seq 1 10); do ready && return 0; sleep 0.3; done
  return 1
}
write_env() {
  printf 'PHONE_IP=%s\nPHONE_PORT=%s\n' "$1" "$2" > "$ENV_FILE"
}

# Load saved IP early so the pairing prompts can reuse it.
SAVED_IP=""
if [ -f "$ENV_FILE" ]; then
  # shellcheck disable=SC1090
  source "$ENV_FILE"
  SAVED_IP="${PHONE_IP:-}"
fi

if ready; then
  log "already connected: $(serial)"
  exit 0
fi

if [ -n "$SAVED_IP" ] && [ -n "${PHONE_PORT:-}" ] && try_connect "$SAVED_IP:$PHONE_PORT"; then
  log "connected via saved address: $(serial)"
  exit 0
fi

for _ in 1 2 3; do
  addr="$(timeout 2 adb mdns services 2>/dev/null | awk '/_adb-tls-connect\._tcp/ {print $NF; exit}')"
  [ -z "$addr" ] && continue
  if try_connect "$addr"; then
    write_env "${addr%:*}" "${addr##*:}"
    log "connected via mDNS: $(serial)"
    exit 0
  fi
done

cat >&2 <<'EOF'

Couldn't connect. This usually means the pairing is gone (phone rebooted,
Wireless Debugging toggled off/on).

On the phone:
  Settings > Developer Options > Wireless Debugging
  Tap "Pair device with pairing code"

That opens a dialog with two values:
  - an IP address and a PORT (this port is unique to pairing)
  - a 6-digit pairing code

You'll enter those next.

EOF

if [ -n "$SAVED_IP" ]; then
  log "using saved IP from $ENV_FILE: $SAVED_IP"
  read -r -p "Pairing port (the number after ':' in the dialog): " pair_port
  [ -z "${pair_port:-}" ] && { log "no port given"; exit 4; }
  pair_addr="$SAVED_IP:$pair_port"
else
  log "no saved IP in $ENV_FILE"
  read -r -p "Pairing IP:port (e.g. 192.168.1.42:41881): " pair_addr
  [ -z "${pair_addr:-}" ] && { log "no address given"; exit 4; }
  if [[ "$pair_addr" != *:* ]]; then
    log "expected ip:port (with a colon), got '$pair_addr'. Use the full value from the pairing dialog, not just the port."
    exit 4
  fi
fi

read -r -p "Pairing code (6 digits from the same dialog): " pair_code
[ -z "${pair_code:-}" ] && { log "no code given"; exit 4; }

log "pairing with $pair_addr..."
paired=0
for attempt in 1 2 3 4; do
  adb pair "$pair_addr" "$pair_code" && { paired=1; break; }
  log "attempt $attempt failed; restarting adb and retrying..."
  adb kill-server 2>/dev/null || true
  sleep 2
done
[ "$paired" -eq 1 ] || { log "pairing failed; open a fresh pairing dialog and re-run"; exit 5; }

cat >&2 <<'EOF'

Paired. Now close the pair dialog and look at the main Wireless Debugging
screen. Under the toggle there's an "IP address & Port" line.

That port is DIFFERENT from the pairing port. Use the one shown here.

EOF

if [ -n "$SAVED_IP" ]; then
  log "using saved IP from $ENV_FILE: $SAVED_IP"
  read -r -p "Connect port (the number after ':' on the main screen): " connect_port
  [ -z "${connect_port:-}" ] && { log "no port given"; exit 6; }
  connect_addr="$SAVED_IP:$connect_port"
else
  read -r -p "Connect IP:port (e.g. 192.168.1.42:5555): " connect_addr
  [ -z "${connect_addr:-}" ] && { log "no address given"; exit 6; }
  if [[ "$connect_addr" != *:* ]]; then
    log "expected ip:port (with a colon), got '$connect_addr'. Use the full value from the main screen, not just the port."
    exit 6
  fi
fi

write_env "${connect_addr%:*}" "${connect_addr##*:}"
log "saved $ENV_FILE"

try_connect "$connect_addr" && { log "connected: $(serial)"; exit 0; }

log "connected to $connect_addr but device did not appear ready"
adb devices >&2
exit 7