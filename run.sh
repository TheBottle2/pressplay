#!/bin/bash
# PressPlay'i root yetkisiyle ama mevcut kullanıcının grafik oturumunda çalıştırır.
# Neden sudo env ... ? : sudo ortam değişkenlerini sıfırlar; DISPLAY / WAYLAND_DISPLAY /
# XDG_RUNTIME_DIR taşınmazsa GUI display server'a bağlanamaz (boş/siyah pencere ya da çökme).
# Kullanım: ./run.sh  (gerekirse: ./run.sh --debug ile debug binary)
set -e
BIN="./target/release/pressplay"
[ "$1" = "--debug" ] && BIN="./target/debug/pressplay"
[ -x "$BIN" ] || { echo "Binary bulunamadı: $BIN (önce cargo build --release)"; exit 1; }
exec sudo env \
  "DISPLAY=$DISPLAY" \
  "WAYLAND_DISPLAY=$WAYLAND_DISPLAY" \
  "XDG_RUNTIME_DIR=$XDG_RUNTIME_DIR" \
  "RUST_LOG=${RUST_LOG:-info}" \
  "$BIN" "$@"
