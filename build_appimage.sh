#!/bin/bash
set -e

echo "=== Linux TinyTask AppImage Builder ==="

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
cp target/release/linux-tinytask "$APPDIR/usr/bin/"
cp linux-tinytask.desktop "$APPDIR/usr/share/applications/"
cp icon.png "$APPDIR/usr/share/icons/hicolor/256x256/apps/linux-tinytask.png"

# 5. AppImage oluştur
echo "AppImage oluşturuluyor..."
./linuxdeploy-x86_64.AppImage \
    --appdir "$APPDIR" \
    --output appimage \
    -e "$APPDIR/usr/bin/linux-tinytask" \
    -d "$APPDIR/usr/share/applications/linux-tinytask.desktop" \
    -i "$APPDIR/usr/share/icons/hicolor/256x256/apps/linux-tinytask.png"

echo ""
echo "✓ AppImage başarıyla oluşturuldu!"
echo "  Çalıştırmak için: chmod +x Linux_TinyTask-*.AppImage && ./Linux_TinyTask-*.AppImage"
echo ""
echo "NOT: Uygulama /dev/input ve /dev/uinput erişimi gerektirir."
echo "     Çalıştırmak için: sudo ./Linux_TinyTask-*.AppImage"
echo "     Veya: sudo usermod -a -G input,uinput \$USER (sonra logout/login)"