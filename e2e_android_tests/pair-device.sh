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

if ready; then
  log "already connected: $(serial)"
  exit 0
fi

if [ -f "$ENV_FILE" ]; then
  # shellcheck disable=SC1090
  source "$ENV_FILE"
  if [ -n "${PHONE_IP:-}" ] && [ -n "${PHONE_PORT:-}" ] && try_connect "$PHONE_IP:$PHONE_PORT"; then
    log "connected via saved address: $(serial)"
    exit 0
  fi
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

No saved address worked and mDNS found nothing. This usually means the
pairing is gone (phone rebooted, Wireless Debugging toggled off/on).

On the phone:
  Settings > Developer Options > Wireless Debugging
  Tap "Pair device with pairing code"
  Copy both values:

EOF
read -r -p "Pairing IP:port: " pair_addr
read -r -p "Pairing code:   " pair_code
[ -z "${pair_addr:-}" ] && { log "no address"; exit 4; }
[ -z "${pair_code:-}" ] && { log "no code"; exit 4; }
[[ "$pair_addr" != *:* ]] && { log "must be host:port"; exit 4; }

log "pairing..."
paired=0
for attempt in 1 2 3 4; do
  adb pair "$pair_addr" "$pair_code" && { paired=1; break; }
  log "attempt $attempt failed; adb kill-server and retry..."
  adb kill-server 2>/dev/null || true
  sleep 2
done
[ "$paired" -eq 1 ] || { log "pairing failed; open a fresh dialog and re-run"; exit 5; }

cat >&2 <<'EOF'

Paired. Now close the pair dialog. On the main Wireless Debugging screen,
under the toggle, copy "IP address & Port".

EOF
read -r -p "Connect IP:port: " connect_addr
[ -z "${connect_addr:-}" ] && { log "no address"; exit 6; }
[[ "$connect_addr" != *:* ]] && { log "must be host:port"; exit 6; }

write_env "${connect_addr%:*}" "${connect_addr##*:}"
log "saved $ENV_FILE"

try_connect "$connect_addr" && { log "connected: $(serial)"; exit 0; }

log "connected to $connect_addr but device did not appear ready"
adb devices >&2
exit 7