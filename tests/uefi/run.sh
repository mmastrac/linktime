#!/usr/bin/env bash
# Boots a UEFI application under QEMU + OVMF and writes its COM1 serial output to
# stdout. The application powers the machine off (ACPI S5) when done, so QEMU
# exits on its own. A watchdog guards against hangs.
#
# Usage: run.sh <path-to-efi>
set -euo pipefail

EFI="${1:?usage: run.sh <path-to-efi>}"

# Locate an OVMF/edk2 firmware pair: read-only code + a vars template we copy so
# QEMU can write it. Paths cover Homebrew (macOS), Debian/Ubuntu and Fedora.
CODE=""
VARS=""
for pair in \
    "/opt/homebrew/share/qemu/edk2-x86_64-code.fd:/opt/homebrew/share/qemu/edk2-i386-vars.fd" \
    "/usr/local/share/qemu/edk2-x86_64-code.fd:/usr/local/share/qemu/edk2-i386-vars.fd" \
    "/usr/share/OVMF/OVMF_CODE_4M.fd:/usr/share/OVMF/OVMF_VARS_4M.fd" \
    "/usr/share/OVMF/OVMF_CODE.fd:/usr/share/OVMF/OVMF_VARS.fd" \
    "/usr/share/edk2/ovmf/OVMF_CODE.fd:/usr/share/edk2/ovmf/OVMF_VARS.fd" \
    "/usr/share/qemu/edk2-x86_64-code.fd:/usr/share/qemu/edk2-i386-vars.fd" \
    ; do
    c="${pair%%:*}"
    v="${pair##*:}"
    if [ -f "$c" ] && [ -f "$v" ]; then
        CODE="$c"
        VARS="$v"
        break
    fi
done

if [ -z "$CODE" ]; then
    echo "run.sh: no OVMF firmware found" >&2
    exit 2
fi

WORK="$(mktemp -d)"
trap 'rm -rf "$WORK"' EXIT
mkdir -p "$WORK/esp/EFI/BOOT"
cp "$EFI" "$WORK/esp/EFI/BOOT/BOOTX64.EFI"
cp "$VARS" "$WORK/vars.fd"

run_qemu() {
    qemu-system-x86_64 \
        -machine q35 \
        -m 256 \
        -drive "if=pflash,format=raw,readonly=on,file=$CODE" \
        -drive "if=pflash,format=raw,file=$WORK/vars.fd" \
        -drive "format=raw,file=fat:rw:$WORK/esp" \
        -display none \
        -serial stdio \
        -net none \
        -no-reboot \
        "$@"
}

# The app powers itself off, so QEMU normally exits on its own. The watchdog is
# just a hang guard, with its fds detached so it can't hold this script's stdout
# pipe open past QEMU's exit.
run_qemu > "$WORK/serial.log" 2>&1 &
QPID=$!
{ sleep 60; kill "$QPID" 2>/dev/null; } >/dev/null 2>&1 &
WPID=$!
wait "$QPID" 2>/dev/null || true
kill "$WPID" 2>/dev/null || true

# OVMF precedes the app with ANSI escapes and boot-manager lines on the same
# serial port. Keep only from the app's first line.
tr -d '\r' < "$WORK/serial.log" | sed -n '/^start$/,$p'
