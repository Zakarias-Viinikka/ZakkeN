#!/usr/bin/env bash
set -u

# Exit codes:
#   0 = exactly one physical device ready (USB or wireless)
#   2 = adb not on PATH
#   3 = no devices visible to adb at all
#   4 = multiple devices ready, or one ready plus extras
#   5 = only emulator(s) ready, no physical device
#   6 = devices present but none in 'device' state (unauthorized/offline/etc.)

if ! command -v adb >/dev/null 2>&1; then
  echo "FAIL: adb not on PATH" >&2
  exit 2
fi

ready_phys=()
ready_emu=()
not_ready=()

while IFS= read -r line; do
  [[ -z "$line" ]] && continue
  serial="${line%%[[:space:]]*}"
  status="${line#"$serial"}"
  status="${status#"${status%%[![:space:]]*}"}"
  status="${status%"${status##*[![:space:]]}"}"

  if [[ "$status" == "device" ]]; then
    case "$serial" in
      emulator-*) ready_emu+=("$serial") ;;
      *)          ready_phys+=("$serial") ;;
    esac
  else
    not_ready+=("$serial ($status)")
  fi
done < <(adb devices | tail -n +2)

if (( ${#ready_phys[@]} == 1 && ${#ready_emu[@]} == 0 && ${#not_ready[@]} == 0 )); then
  echo "OK: one physical device ready: ${ready_phys[0]}"
  exit 0
fi

if (( ${#ready_phys[@]} == 0 && ${#ready_emu[@]} == 0 && ${#not_ready[@]} == 0 )); then
  echo "NONE: no devices visible to adb."
  echo "  USB: check cable, unlock the phone, accept the USB debugging prompt."
  echo "  Wireless: run 'adb pair <ip>:<pair-port>' (Android 11+) then 'adb connect <ip>:<port>', or 'adb tcpip 5555' + 'adb connect <ip>:5555' on older."
  exit 3
fi

echo "Devices seen by adb:"
for d in "${ready_phys[@]}"; do echo "  - $d (physical, ready)"; done
for d in "${ready_emu[@]}";  do echo "  - $d (emulator, ready)"; done
for d in "${not_ready[@]}";  do echo "  - $d"; done

if (( ${#ready_phys[@]} == 0 && ${#ready_emu[@]} > 0 )); then
  echo "FAIL: only emulator(s), no physical device."
  exit 5
fi

if (( ${#ready_phys[@]} > 1 )); then
  echo "FAIL: multiple physical devices ready; disconnect all but one."
  exit 4
fi

if (( ${#ready_phys[@]} == 1 )); then
  echo "FAIL: one physical ready but other devices are also present; disconnect them."
  exit 4
fi

echo "FAIL: no ready physical device."
exit 6
