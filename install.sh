#!/bin/bash
# One-time setup: build release, install binary + menu entry + icon,
# grant input/uinput access (no sudo needed at launch afterwards).
# Usage: ./install.sh
# Afterwards: launch "Linux TinyTask" from the app menu (log out/in first if groups changed).
set -e
cd "$(dirname "$0")"

echo "=== Linux TinyTask installer ==="

echo "[1/4] Building release..."
cargo build --release

PREFIX="${HOME}/.local"
mkdir -p "$PREFIX/bin" "$PREFIX/share/applications" "$PREFIX/share/icons/hicolor/256x256/apps"

echo "[2/4] Installing files to $PREFIX..."
install -m755 "target/release/linux-tinytask" "$PREFIX/bin/linux-tinytask"
install -m644 "linux-tinytask.desktop" "$PREFIX/share/applications/"
install -m644 "icon.png" "$PREFIX/share/icons/hicolor/256x256/apps/linux-tinytask.png"
update-desktop-database "$PREFIX/share/applications" 2>/dev/null || true

echo "[3/4] Input permissions (one-time, may ask for sudo)..."
NEED_RELOGIN=0
if ! groups "$USER" 2>/dev/null | grep -qw input; then
    sudo usermod -aG input "$USER"
    NEED_RELOGIN=1
fi
RULE_FILE="/etc/udev/rules.d/99-tinytask-uinput.rules"
if [ ! -f "$RULE_FILE" ]; then
    echo 'KERNEL=="uinput", GROUP="input", MODE="0660"' | sudo tee "$RULE_FILE" > /dev/null
    sudo udevadm control --reload-rules
    sudo udevadm trigger --subsystem-misc 2>/dev/null || true
fi

echo "[4/4] Done."
if [ "$NEED_RELOGIN" = "1" ]; then
    echo "  -> You were added to the 'input' group. LOG OUT and back in once,"
    echo "     then launch 'Linux TinyTask' from the app menu. No sudo needed."
else
    echo "  -> Launch 'Linux TinyTask' from the app menu. No sudo needed."
fi
