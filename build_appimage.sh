#!/bin/bash
set -e

echo "=== PressPlay AppImage Builder ==="

# 1. Release build
echo "Release build yapılıyor..."
cargo build --release

# 2. AppImage araçlarını indir
if [ ! -f "linuxdeploy-x86_64.AppImage" ]; then
    echo "linuxdeploy indiriliyor..."
    wget -q https://github.com/linuxdeploy/linuxdeploy/releases/download/continuous/linuxdeploy-x86_64.AppImage
    chmod +x linuxdeploy-x86_64.AppImage
fi

# 3. AppDir yapısını oluştur
APPDIR="AppDir"
rm -rf "$APPDIR"
mkdir -p "$APPDIR/usr/bin"
mkdir -p "$APPDIR/usr/share/applications"
mkdir -p "$APPDIR/usr/share/icons/hicolor/256x256/apps"

# 4. Binary ve asset'leri kopyala
cp target/release/pressplay "$APPDIR/usr/bin/"
cp pressplay.desktop "$APPDIR/usr/share/applications/"
cp icon.png "$APPDIR/usr/share/icons/hicolor/256x256/apps/pressplay.png"

# 5. AppImage oluştur
echo "AppImage oluşturuluyor..."
./linuxdeploy-x86_64.AppImage \
    --appdir "$APPDIR" \
    --output appimage \
    -e "$APPDIR/usr/bin/pressplay" \
    -d "$APPDIR/usr/share/applications/pressplay.desktop" \
    -i "$APPDIR/usr/share/icons/hicolor/256x256/apps/pressplay.png"

echo ""
echo "✓ AppImage başarıyla oluşturuldu!"
echo "  Çalıştırmak için: chmod +x PressPlay-*.AppImage && ./PressPlay-*.AppImage"
echo ""
echo "NOT: Uygulama /dev/input ve /dev/uinput erişimi gerektirir."
echo "     Çalıştırmak için display env ile sudo ./PressPlay-*.AppImage"
echo "     Veya: ./install.sh (önerilen, sudo'suz başlatma)"