# Linux TinyTask

**Linux için minimalist, kernel-seviyesi makro kaydedici ve oynatıcı**

Linux TinyTask, Windows'taki popüler TinyTask uygulamasının Linux karşılığıdır. X11 ve Wayland'dan bağımsız olarak doğrudan Linux kernel input subsystem (`/dev/input` + `/dev/uinput`) ile çalışır, milisaniye/mikrosaniye hassasiyetinde kayıt ve oynatma yapar.

## 🎯 Özellikler

### Çalışan Özellikler
- **Kernel-seviyesi kayıt**: `/dev/input/event*` üzerinden tüm klavye + mouse (REL) yakalama, 1ms `poll` timeout
- **Kernel-seviyesi oynatma**: `/dev/uinput` sanal cihaz (`LinuxTinyTask Virtual Device`) üzerinden event enjeksiyonu
- **Display server bağımsızlığı**: X11/Wayland/Proton/Wine'da çalışır, X11/Wayland kütüphanesine bağımlı değildir
- **Hassas zamanlama**: deadline tabanlı kesilebilir `precise_sleep_interruptible` (100µs altı busy-wait, üstü 2ms parçalı sleep + son 800µs spin), event başına `timestamp_us` farkı kadar bekleme
- **Acil durdurma**: `AtomicBool` stop bayrağı; her event öncesi + sleep içinde ~1ms'de bir kontrol edilir, `StopPlayback` anında emit'i keser, **basılı kalmış tuşları otomatik bırakır** (takılı Ctrl vb. yüzünden klavye/fare bozulmaz) ve Idle'a geçirir (oynatma içi kanal yoklama ile)
- **Makro dosyaları**: `.tts` (bincode, küçük/hızlı) ve `.json` (okunabilir/debug) formatlarında kaydetme/yükleme; versiyonlu `MacroFile` sarmalayıcı, path-traversal korumalı
- **Makro paneli**: Kontrol sekmesinde hızlı Kaydet/Yükle + ayrı `Makrolar` sekmesi (ad, süre, event sayısı, tarih)
- **Süre gösterimi**: kayıtta `duration_us` hesaplanır, UI'da `12.34s / 850ms / 400µs` formatında gösterilir
- **Döngü modu**: 1–9999 tekrar veya sonsuz döngü (`0 = sonsuz`), döngü arası 50ms bekleme
- **Global hotkey'ler**: Kayıt/oynat/durdur için sistem geneli kısayollar (kayıt thread'inden bağımsız hotkey thread)
- **Config persistence**: Hotkey yapılandırması `~/.config/linux-tinytask/tinytask_config.json` içinde JSON olarak saklanır
- **Minimalist UI**: `eframe/egui` ile always-on-top, 420x480, sekmeli arayüz (Kontrol / Makrolar / Ayarlar / Hakkında)
- **Çok thread'li mimari**: Dispatcher + Recorder + Player + Sync + Hotkey + UI thread'leri, `crossbeam-channel` ile haberleşme

### Varsayılan Kısayollar
| İşlem | Varsayılan |
|---|---|
| Kayıt Başlat/Durdur (toggle) | `Ctrl+Alt+Shift+R` |
| Oynat Başlat | `Ctrl+Alt+Shift+P` |
| Oynatmayı Durdur | `Ctrl+Alt+Shift+S` |

Kısayollar **Ayarlar** sekmesinden değiştirilebilir: `Değiştir` → tek tuş (örn. `F8`) veya `Ctrl/Alt/Shift` ile birlikte bir tuş basın. Atama anında aktif olur ve diske kaydedilir.
> ⚠ Tek harf/tuş kısayollar yazı yazarken de tetiklenir — `F8–F12` önerilir. `Super` tuşu yakalanamaz (egui bildirmez), mevcut Super'li kısayollar çalışmaya devam eder.

> Not: `KeyCombo::new()` varsayılan olarak `ctrl+alt+shift` basılı + ana tuş ister. Eşleşme `KeyCombo::matches()` ile sol/sağ Ctrl/Alt/Shift/Super kodlarına bakılarak yapılır. 200ms debounce vardır.

## 🏗️ Mimari

```
UI (egui) ──Command──▶ Dispatcher ──┬──▶ Recorder ──▶ /dev/input/event* (poll 1ms)
                                    │         ↕ (Arc<Mutex<MacroRecording>>)
                                    │     Sync thread (5ms poll, Recording→Idle geçişinde kopyalar)
                                    │         ↓
                                    └──▶ Player ──▶ /dev/uinput (VirtualDevice)
Hotkey thread (/dev/input poll) ──Command──▶ Dispatcher
Recorder/Player ──String──▶ UI (status_message)
```

### Dosya Yapısı
```
linux-tinytask/
├── src/
│   ├── main.rs      # Config load/save, dispatcher, sync thread, hotkey thread, thread spawn
│   ├── models.rs    # MacroEvent, MacroRecording, AppState, KeyCombo, HotkeyConfig, Command
│   ├── recorder.rs  # /dev/input enumeration + poll + kayıt (SYN hariç tüm eventler)
│   ├── player.rs    # uinput sanal cihaz + precise_sleep + loop oynatma
│   └── ui.rs        # eframe/egui: Kontrol / Ayarlar / Hakkında sekmeleri
├── Cargo.toml
├── Cargo.lock
├── build_appimage.sh
├── linux-tinytask.desktop
├── icon.png / icon.svg
├── README.md
└── HANDOFF.md       # Oturumlar arası devir dosyası (her işlem sonrası güncellenir)
```

### Veri Modeli
- `MacroEvent { timestamp_us: u64, event_type: u16, code: u16, value: i32 }` — `from_evdev`/`to_evdev` ile dönüşüm.
- `MacroRecording { name, created_at, duration_us, events: Vec<MacroEvent> }` — 10.000 kapasiteyle başlar.
- `Command`: `StartRecording | StopRecording | StartPlayback | StopPlayback | SaveMacro(String) | LoadMacro(String) | SetHotkey(HotkeyAction, KeyCombo) | SetLoopCount(u32) | SaveConfig | Quit`
- `AppState`: `Idle | Recording | Playing` — `Arc<Mutex<...>>` ile tüm thread'ler arasında paylaşılır.

## 📋 Gereksinimler

### Sistem
- Linux kernel 4.x+, x86_64
- X11 veya Wayland (fark etmez)
- `/dev/input` okuma ve `/dev/uinput` yazma izni (aşağıya bak)

### Build
- Rust 1.70+ + Cargo
- gcc/make (build essentials)
- `libudev-dev` (Debian/Ubuntu) veya `systemd-devel` (Fedora)

### Bağımlılıklar (`Cargo.toml`)
`evdev 0.12`, `nix 0.28 (poll, fs)`, `eframe/egui 0.27`, `serde + serde_json`, `bincode 1.3` (.tts formatı), `chrono 0.4`, `log + env_logger`, `crossbeam-channel 0.5`, `dirs 5.0`, `rfd 0.14` (native dosya dialogu).

## 🚀 Kurulum

### 1. Rust Kurulumu
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env
rustup default stable
```

### 2. Sistem Bağımlılıkları
```bash
# Debian/Ubuntu
sudo apt install build-essential libudev-dev

# Fedora
sudo dnf install gcc make systemd-devel
```

### 3. İzinleri Ayarlama (zorunlu)
Uygulama `/dev/input` ve `/dev/uinput` erişimi olmadan çalışmaz. İkisinden birini seç:

```bash
# Seçenek A: gruplara ekle (önerilen, kalıcı)
sudo usermod -a -G input,uinput $USER
# sonra logout/login veya reboot

# Seçenek B: tek seferlik root ile çalıştır
sudo ./target/release/linux-tinytask
```
İzin yoksa log'da şunu görürsün: `Cannot read /dev/input`, `Sanal cihaz oluşturulamadı`, `Hiçbir input cihazı bulunamadı!`.

### 4. Derleme ve Çalıştırma
```bash
cargo build --release
./target/release/linux-tinytask
# log seviyesi: RUST_LOG=debug ./target/release/linux-tinytask
```

### 4b. Root ile çalıştırma (GUI + sudo)
Çıplak `sudo` ortamı sıfırlar ve GUI display'e bağlanamaz. Ya grupları kullanın
(Seçenek A, önerilen) ya da display değişkenlerini taşıyın:
```bash
./run.sh                                   # taze release binary ile, display korunarak sudo
sudo env "DISPLAY=$DISPLAY" "WAYLAND_DISPLAY=$WAYLAND_DISPLAY" \
  "XDG_RUNTIME_DIR=$XDG_RUNTIME_DIR" ./target/release/linux-tinytask
```
Not: root ile çalışınca config `/root/.config/linux-tinytask/` altına yazılır.

### 5. AppImage (taşınabilir)
```bash
chmod +x build_appimage.sh
./build_appimage.sh
chmod +x Linux_TinyTask-*.AppImage
sudo ./Linux_TinyTask-*.AppImage   # UYARI: çıplak sudo GUI'yi bozar; display env'leri taşıyın:
sudo env "DISPLAY=$DISPLAY" "WAYLAND_DISPLAY=$WAYLAND_DISPLAY" \
  "XDG_RUNTIME_DIR=$XDG_RUNTIME_DIR" ./Linux_TinyTask-*.AppImage
```

## 🖥️ Kullanım

1. Uygulamayı başlat (pencere her zaman üstte).
2. **Kontrol** sekmesi → `● Kaydet` (veya `Ctrl+Alt+Shift+R`) → işlemleri yap → `■ Durdur`.
3. Döngü ayarı: `Sonsuz döngü` işaretle ya da `1–9999` seç → `Döngü Ayarını Uygula` (gönderilmezse varsayılan `1` kullanılır).
4. `▶ Oynat` (veya `Ctrl+Alt+Shift+P`) → durdurmak için oynatma sırasındaki **iki butondan herhangi biri** (veya `Ctrl+Alt+Shift+S`). Durdurma, kanal gecikmesinden bağımsız paylaşılan atomic bayrakla anında emit'i keser.
5. Durum satırı ve `Kaydedilen event` sayacı o anki durumu gösterir.

### Config Dosyası
Yol: `~/.config/linux-tinytask/tinytask_config.json`
Örnek:
```json
{
  "record": { "ctrl": true, "alt": true, "shift": true, "super_key": false, "key_code": 19 },
  "play":   { "ctrl": true, "alt": true, "shift": true, "super_key": false, "key_code": 25 },
  "stop":   { "ctrl": true, "alt": true, "shift": true, "super_key": false, "key_code": 31 }
}
```
Anahtar kodlar Linux evdev kodlarıdır (19=R, 25=P, 31=S, 1=Esc, 57=Space vb. — tam liste `models.rs` içinde).

### Makro Dosyaları
- Kontrol sekmesindeki `💾 Kaydet` / `📂 Yükle` veya `Makrolar` sekmesi kullanılır (`rfd` native dialog).
- Uzantıya göre format: `.json` → insan-okunabilir JSON, `.tts` (veya diğer) → `bincode` binary (küçük/hızlı).
- Dosya yapısı: `MacroFile { version: 1, name, created_at, duration_us, event_count, events }`. Dosya IO recorder thread'de yapılır (UI bloklanmaz); yüklenen makro player kopyasına anında senkronize edilir.
- Test: `cargo test` (7 test: json/binary roundtrip, path-traversal reddi, tek-tuş eşleşme, sleep doğruluğu, stop tepkisi, tuş takibi).

## ⚠️ Bilinen Eksikler / Sınırlılıklar

Kodun güncel durumuna göre dürüst liste (detay `HANDOFF.md` içinde):

1. **ABS (absolute) eksen oynatılmıyor**: Recorder `ABS` eventlerini kaydeder ama `player.rs` sanal cihazı sadece tuş + `REL_X/Y/WHEEL/HWHEEL` açar. Grafik tablet/touchscreen mutlak konumları oynatılamaz.
2. **Sync thread kırılgan**: `Recording → Idle` geçişini 5ms'de bir poll ederek yakalar; hızlı toggle veya boş kayıt edge-case'lerinde yarış olabilir.
3. **Kapanış temiz değil**: UI kapanınca `std::process::exit(0)` ile sert çıkış yapılır; thread'lere `Quit` gönderilmez. (Player içi Quit artık oynatma sırasında da işleniyor.)
4. **Hotkey thread'de çıkış yok**: Sonsuz `loop`, `Quit` dinlemez; power/video/lid filtreler ama yine de tüm klavyeleri dinler.
5. **Hotkey tuşları kayda karışır**: Kayıt sırasında hotkey'e basılan tuşlar filtrelenmez.
6. Kayıt sırasında hotkey'e basılan tuşlar kayda karışır (kayda UI butonuyla başlanırsa sorun olmaz).

## 🛣️ Yol Haritası
- [x] Makro kaydet/yükle (JSON + bincode): dosya dialogu + `SaveMacro/LoadMacro` implementasyonu
- [x] Acil durdurma + zamanlama düzeltmesi (kesilebilir sleep, oynatma-içi kanal yoklama)
- [x] Süre hesaplama + UI gösterimi
- [x] Makro yönetim paneli (Makrolar sekmesi)
- [x] Hotkey atama (tek tuş dahil: Ayarlar → Değiştir → tuşa bas; diske kaydedilir)
- [x] Stop/finish'te takılı tuşları otomatik bırakma (bozuk klavye/fare düzeltmesi)
- [ ] ABS eksen + `REL_Z` vb. için sanal cihaz genişletmesi
- [ ] Düzgün shutdown (`Quit` yayılımı, `process::exit` kaldırma)
- [ ] Kayıt sırasında hotkey'e basılan tuşların kayda karışmaması
- [ ] Oynatma hız çarpanı, gecikme düzenleme

## 🧪 Test Senaryoları (talepten)
1. **Durdurma**: 10sn makro kaydet → oynat → 2. sn'de durdur. Beklenen: emit anında kesilir, kalan eventler oynatılmaz, durum Idle. (Manuel: `/dev/input`+`uinput` izinli gerçek makinede.)
2. **Zamanlama**: 5sn makro → oynat → toplam süre 5s ±50ms olmalı. Not: ilk event öncesi bekleme de kayda dahildir.
3. **Loop durdurma**: sonsuz döngü → durdur → yeni loop başlamamalı, `Playback stopped` logu.
4. **Kaydet/Yükle**: kaydet → dosyaya yaz → kapat/aç → yükle → oynat. Otomatik: `cargo test` (7 test geçiyor).
5. **Büyük makro**: 10.000 event roundtrip — `cargo test` + manuel kayıt ile doğrulanmalı.

## 🐛 Sorun Giderme
| Belirti | Neden / Çözüm |
|---|---|
| `Cannot read /dev/input` | `input` grubunda değilsin → `usermod -a -G input $USER` + relogin |
| `Sanal cihaz oluşturulamadı` | `uinput` izni yok → `usermod -a -G uinput $USER` veya `sudo` ile çalıştır; `ls -l /dev/uinput` kontrol et |
| `Oynatılacak event yok!` | Kayıt boş → önce kayıt yap veya Makrolar sekmesinden dosya yükle (kaydedilmemiş kayıt restart'ta sıfırlanır) |
| Hotkey çalışmıyor | Başka uygulama tuşu yutuyor olabilir; terminalden `RUST_LOG=debug` ile basılan kodları gözle |
| Wayland'da çalışmıyor | İzin sorunudur, display server ile ilgili değildir — grupları kontrol et |

## 📄 Lisans
MIT — detay için `LICENSE` dosyasına bakın.
