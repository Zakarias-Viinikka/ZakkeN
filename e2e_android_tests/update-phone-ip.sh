#!/usr/bin/env bash
set -uo pipefail
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ENV_FILE="$SCRIPT_DIR/ip.env"

log() { echo "[update-phone-ip] $*" >&2; }

if [ -f "$ENV_FILE" ]; then
  # shellcheck disable=SC1090
  source "$ENV_FILE"
fi
cur_ip="${PHONE_IP:-}"
cur_port="${PHONE_PORT:-}"

write_env() {
  printf 'PHONE_IP=%s\nPHONE_PORT=%s\n' "$1" "$2" > "$ENV_FILE"
}

connected_addr() {
  adb devices 2>/dev/null | awk 'NR>1 && $2=="device" && $1 ~ /:[0-9]+$/ {print $1; exit}'
}

# 1. Auto-detect from a live adb connection.
addr="$(connected_addr)"
if [ -n "$addr" ]; then
  write_env "${addr%:*}" "${addr##*:}"
  log "detected live connection: $addr"
  log "saved to $ENV_FILE"
  exit 0
fi

# 2. Args: ./update-phone-ip.sh <IP> [PORT]
if [ $# -ge 1 ]; then
  new_ip="$1"
  new_port="${2:-$cur_port}"
  [[ "$new_ip" != *.*.*.* ]] && { log "invalid IP: $new_ip"; exit 2; }
  [ -z "$new_port" ] && { log "port required"; exit 3; }
  write_env "$new_ip" "$new_port"
  log "saved: $new_ip:$new_port"
  exit 0
fi

# 3. Interactive with defaults pre-filled.
[ ! -t 0 ] && { log "not connected, no args, and no terminal to prompt"; exit 4; }
read -r -p "Phone IP [${cur_ip:-new}]: " input_ip
new_ip="${input_ip:-$cur_ip}"
[ -z "$new_ip" ] && { log "IP required"; exit 5; }
read -r -p "Port [${cur_port:-new}]: " input_port
new_port="${input_port:-$cur_port}"
[ -z "$new_port" ] && { log "port required"; exit 6; }

write_env "$new_ip" "$new_port"
log "saved: $new_ip:$new_port"
