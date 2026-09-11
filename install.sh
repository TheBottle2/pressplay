#!/bin/bash
# One-time setup: build release, install binary + menu entry + icon,
# grant input/uinput access (no sudo needed at launch afterwards).
# Also removes legacy linux-tinytask installs (pre-0.2.0 name).
# Usage: ./install.sh
# Afterwards: launch "PressPlay" from the app menu (log out/in first if groups changed).
set -e
cd "$(dirname "$0")"

echo "=== PressPlay installer ==="

echo "[1/5] Building release..."
cargo build --release

PREFIX="${HOME}/.local"
mkdir -p "$PREFIX/bin" "$PREFIX/share/applications" "$PREFIX/share/icons/hicolor/256x256/apps"

echo "[2/5] Removing legacy linux-tinytask install (if any)..."
rm -f "$PREFIX/bin/linux-tinytask"
rm -f "$PREFIX/share/applications/linux-tinytask.desktop"
rm -f "$PREFIX/share/icons/hicolor/256x256/apps/linux-tinytask.png"
if [ -f /etc/udev/rules.d/99-tinytask-uinput.rules ]; then
    sudo rm -f /etc/udev/rules.d/99-tinytask-uinput.rules
fi

echo "[3/5] Installing files to $PREFIX..."
install -m755 "target/release/pressplay" "$PREFIX/bin/pressplay"
install -m644 "pressplay.desktop" "$PREFIX/share/applications/"
install -m644 "icon.png" "$PREFIX/share/icons/hicolor/256x256/apps/pressplay.png"
update-desktop-database "$PREFIX/share/applications" 2>/dev/null || true

echo "[4/5] Input permissions (one-time, may ask for sudo)..."
NEED_RELOGIN=0
if ! groups "$USER" 2>/dev/null | grep -qw input; then
    sudo usermod -aG input "$USER"
    NEED_RELOGIN=1
fi
RULE_FILE="/etc/udev/rules.d/99-pressplay-uinput.rules"
if [ ! -f "$RULE_FILE" ]; then
    echo 'KERNEL=="uinput", GROUP="input", MODE="0660"' | sudo tee "$RULE_FILE" > /dev/null
    sudo udevadm control --reload-rules
    sudo udevadm trigger --subsystem-misc 2>/dev/null || true
fi

echo "[5/5] Done."
if [ "$NEED_RELOGIN" = "1" ]; then
    echo "  -> You were added to the 'input' group. LOG OUT and back in once,"
    echo "     then launch 'PressPlay' from the app menu. No sudo needed."
else
    echo "  -> Launch 'PressPlay' from the app menu. No sudo needed."
fi
