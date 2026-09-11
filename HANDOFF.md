# HANDOFF.md — Linux TinyTask

> **Bu dosyanın kuralı:** Her işlem (kod değişikliği, hata düzeltme, araştırma, doc güncellemesi) bittiğinde bu dosya güncellenmek ZORUNDADIR.
> Güncelleme formatı: `Son Güncelleme` tarihini değiştir + `Geçmiş` tablosuna satır ekle + etkilenen bölümleri (`Durum`, `Sonraki Adımlar`, `Teknik Borç`) düzelt.
> Dosya Türkçe tutulur.

- **Son Güncelleme:** 2026-09-09
- **Proje:** linux-tinytask v0.1.0 (`/mnt/harddisk/my_apps/linux-tinytask`)
- **Dil/Stack:** Rust 2021, evdev 0.12, nix 0.28, eframe/egui 0.27, crossbeam-channel 0.5, serde_json, bincode 1.3, rfd 0.14
- **Durum:** Kayıt (hotkey-filtreli, klavye/fare seçmeli) + kesilebilir oynatma (hız çarpanlı, takılı-tuş bırakmalı) + döngü + hotkey (tek tuş atanabilir) + dosya Save/Load + son-dosyalar + 9 dil + temiz kapanış çalışıyor. `cargo check` temiz, `cargo test` 16/16, release taze derlendi. Gerçek donanım testleri (hız, filtre, kapanış) manuel bekliyor — **her düzeltmeden sonra release da derlenmeli, kullanıcı release çalıştırıyor. Çalıştırma: `./run.sh`.**

## 1. Sistem Özeti (kısa)

Kernel-seviyesi makro kaydedici/oynatıcı. 6 thread: UI (egui) → Dispatcher (kanal yönlendirici) → Recorder (`/dev/input` poll 1ms) / Player (`/dev/uinput` + `precise_sleep`) + Sync (Recording→Idle geçişinde `MacroRecording.events` kopyalar, 5ms poll) + Hotkey (`/dev/input` KEY dinler, 200ms debounce).
Paylaşılan durum: `Arc<Mutex<AppState>>`, `Arc<Mutex<MacroRecording>>`, `Arc<Mutex<HotkeyConfig>>`, `Arc<Mutex<Vec<MacroEvent>>>` (player kopyası), `Arc<Mutex<u32>>` loop sayacı.
Config: `~/.config/linux-tinytask/tinytask_config.json`. Varsayılan hotkey'ler `Ctrl+Alt+Shift+R/P/S` (evdev kodları 19/25/31).

Dosya haritası:
- `src/main.rs` — config load/save, dispatcher, sync, hotkey thread, spawn (kanal bufferları 50'ye çıkarıldı; Recorder'a player_events paylaşıldı)
- `src/models.rs` — MacroEvent/MacroRecording/AppState/KeyCombo/HotkeyConfig/Command + `MacroFile {version,name,created_at,duration_us,event_count,events}` + `save_to_file/load_from_file` (uzantıya göre JSON/bincode, `..` korumalı) + `duration_string/format_duration` + 3 unit test
- `src/recorder.rs` — cihaz enumeration (power/video/lid hariç), SYN atlanır, local buffer → ana recording; `SaveMacro/LoadMacro` IO burada (Load sonrası player kopyasına anında sync); Stop mesajı `N events, SÜRE` içerir
- `src/player.rs` — VirtualDevice (tuş 0..768 + REL_X/Y/WHEEL/HWHEEL), loop (0=sonsuz), döngü arası 50ms; kesilebilir sleep + oynatma-içi kanal yoklama + `AtomicBool` acil stop; **stop/finish'te `track_held`/`release_held` ile basılı tuşları otomatik bırakır (stuck-modifier düzeltmesi)**
- `src/ui.rs` — Kontrol/Makrolar/Ayarlar/Hakkında (420x480); Kontrol'de süre satırı + hızlı Kaydet/Yükle; Ayarlar'da **gerçek tuş yakalama** (`CAPTURE_KEYS` + `egui_key_to_evdev`): tek tuş (F8 vb.) veya Ctrl/Alt/Shift'li kombo, paylaşılan config Arc'ine + `crate::save_config` ile diske yazılır (hotkey thread anında görür); Playing'de iki buton da `request_stop()`; **tüm metinler `i18n::t()` anahtarlı + dil seçici**
- `Cargo.toml` — `rfd 0.14` eklendi; `build_appimage.sh`, `linux-tinytask.desktop` (Icon mutlak yol sorunu sürüyor), `Cargo.toml`

## 2. Yapılanlar (kümülatif özet)
- 2026-09-09 — README baştan yazıldı (önceki 46. satırda kesiliyordu): gerçek mimari, veri modeli, kurulum+izin adımları, config örneği, dürüst "Bilinen Eksikler" (7 madde), yol haritası, sorun giderme tablosu eklendi. Kod değiştirilmedi.
- 2026-09-09 — Bu HANDOFF.md oluşturuldu.
- 2026-09-09 — Hata/özellik paketi (talepteki 5 madde):
  - Sorun 1 (durdurma): Player iç döngüde kanal yoklamıyordu → her event öncesi `poll_commands` + `AtomicBool` + kesilebilir sleep + emit-öncesi stop kontrolü eklendi (`player.rs`). Kanal bufferları 10→50 (`main.rs`).
  - Sorun 2 (zamanlama): `precise_sleep` %90 sleep + TAM sürenin busy-wait'i = ~1.9x yavaşlıyordu → deadline tabanlı `precise_sleep_interruptible` ile düzeltildi (`player.rs`).
  - Sorun 3 (süre): `duration_us` zaten hesaplanıyordu ama UI'da yoktu → `duration_string/format_duration` + Kontrol satırı + Stop/Load mesajlarına süre eklendi (`models.rs`, `recorder.rs`, `ui.rs`).
  - Özellik 1 (dosya): `MacroFile` (v1) + `save_to_file/load_from_file` (`.json`/`.tts`, `..` korumalı, eski saf-Vec bincode geriye uyumlu) + `rfd` dialog + IO recorder thread'de (`models.rs`, `recorder.rs`, `ui.rs`, `Cargo.toml`).
  - Özellik 2 (panel): `Makrolar` sekmesi + Kontrol'e hızlı Kaydet/Yükle (`ui.rs`).
  - Doğrulama: `cargo check` temiz (tek pre-existing `save_config` warning), `cargo build` OK, `cargo test` 3/3 geçti, headless çalıştırma threadsiz-crash yok. Gerçek donanım testleri (10sn durdurma, 5sn ±50ms, sonsuz loop stop, 10k event) manuel bekliyor.

## 3. Teknik Borç / Bilinen Hatalar (öncelikli, güncel)

1. Player ABS desteklemez (`player.rs` sanal cihaz); recorder ABS kaydeder → tablet/touch mutlak konum oynatılamaz.
2. Sync thread yarışa açık (`main.rs` 5ms poll); Load sonrası sync recorder içinde manuel yapılıyor ama kayıt-yarışı sürüyor.
3. Hotkey-filtre kenar durumu: tetik tuşu + chord modifier'ları kayıttan ayıklanır, ama kayda *başlamadan önce* basılı tutulan bir modifier da chord'u tamamlarsa ayıklanabilir (nadir; release baskılı olduğu için takılı tuş oluşmaz).
4. Zamanlama notu: `device.emit` event başına 1 syscall; ultra-yoğun makrolarda batch/SYN optimizasyonu gerekebilir. İlk-event-öncesi bekleme kayda dahil (tasarım).
5. egui Super tuşunu bildirmez → Super'li kombo yakalanamaz (mevcut Super'li config çalışmaya devam eder).
6. Kapanış duman testi headless yapılamadı (ortamda display var, UI pencere açıp bekliyor) — gerçek kapatma testi kullanıcıda: pencereyi kapatınca ~1sn içinde `All threads stopped` ile çıkmalı.

## 4. Sonraki Adımlar (önerilen sıra)

- [x] Save/Load implementasyonu ~~→ borç #1 kapatır~~ (2026-09-09 yapıldı)
- [x] Acil stop + zamanlama + süre gösterimi + Makrolar sekmesi (2026-09-09 yapıldı)
- [x] Ölü stop butonu düzeltmesi + paylaşılan stop bayrağı + release derleme (2026-09-09)
- [x] Takılı-tuş bırakma (stop/finish) + tek-tuş dahil hotkey yakalama (2026-09-09)
- [x] 5'li paket (2026-09-11): hız çarpanı + hotkey filtreleme + temiz kapanış + son dosyalar + kayıt filtresi
- [ ] Player'a ABS eksen ekleme
- [ ] Manuel donanım testleri: hız (1x/2x), hotkey filtresi (F8), kapanış (`All threads stopped`), 10k event Save/Load/Play

## 5. Faydalı Komutlar

```bash
cargo check
cargo build --release   # kullanıcı bunu çalıştırıyor; düzeltmeden sonra ŞART
./run.sh                # display env'lerini koruyarak sudo+GUI (önerilen çalıştırma)
# manuel eşdeğeri: sudo env "DISPLAY=$DISPLAY" "WAYLAND_DISPLAY=$WAYLAND_DISPLAY" "XDG_RUNTIME_DIR=$XDG_RUNTIME_DIR" ./target/release/linux-tinytask
cargo test   # 16 test: roundtrip, filtre, hotkey/chord, recent, sleep, stop, tuş takibi, hız, i18n (4)
./target/release/linux-tinytask
RUST_LOG=debug ./target/release/linux-tinytask
./build_appimage.sh
ls -l /dev/uinput; groups $USER; cat ~/.config/linux-tinytask/tinytask_config.json
```

## 6. Geçmiş (her işlemde satır ekle)

| Tarih | İşlem | Sonuç / Not |
|---|---|---|
| 2026-09-09 | README sisteme göre düzenlendi | 46 satırlık yarım dosya → tam doc; kod değişikliği yok |
| 2026-09-09 | HANDOFF.md oluşturuldu | Sürekli güncellenecek devir dosyası; kural başlıkta tanımlı |
| 2026-09-09 | Hata/özellik paketi (5 madde) | Stop acil-durdurma, ~1.9x sleep bug düzeltmesi, süre UI, .tts/.json Save/Load, Makrolar sekmesi; `cargo test` 3/3; README+HANDOFF güncellendi |
| 2026-09-09 | "Durdurma butonu çalışmıyor" şikayeti | 2 kök neden: (a) Playing'de sol buton "■ Durdur" yazıp disabled idi — artık iki buton da `request_stop()`; (b) düzeltmeler sadece debug'a derlenmişti — release yeniden derlendi. Ek sağlamlık: paylaşılan `stop_flag` (UI+hotkey→Player, kanal bağımsız), dispatcher `try_send`. Yeni testler: sleep_timing + sleep_interrupted (5/5 geçti) |
| 2026-09-09 | sudo+GUI çalıştırma düzeltmesi | Çıplak `sudo` display'e bağlanamaz (env sıfırlanır). Kullanıcının komutu doğrulandı: `sudo env DISPLAY/WAYLAND_DISPLAY/XDG_RUNTIME_DIR ...`. `run.sh` wrapper eklendi, README (4b + AppImage notu) güncellendi. Not: root ile config `/root/.config/` altına yazılır |
| 2026-09-09 | Takılı-tuş + tek-tuş paketi | (a) Stop sonrası bozuk klavye/fare: key-down'da kesilen oynatma tuşu basılı bırakıyordu → `track_held`/`release_held` ile stop+finish'te otomatik key-up (durum mesajında "N tuş bırakıldı"). (b) Tek-tuş hotkey: Ayarlar'da gerçek tuş yakalama (`CAPTURE_KEYS`, F8 vb. tek tuş veya modifier'lı), diske kayıt, F-isim gösterimi. `cargo test` 7/7, release 23:46 derlendi |
| 2026-09-11 | v0.1.0 sürüm kopyası | Çalışan kod donduruldu: `/mnt/harddisk/my_apps/linux-tinytask-v0.1.0/` (src + kök dosyalar + release binary, `target/` hariç; diff ile doğrulandı). Bozulan bir değişiklikte bu dizinden geri dönülecek |
| 2026-09-11 | GitHub hazırlığı | `.desktop` Icon→`linux-tinytask`, `.gitignore`, sır taraması temiz, MIT `LICENSE`, README lisans bölümü. `git init -b main` + ilk commit `36a07c8` (16 dosya, target hariç). Hedef: `github.com/TheBottle2/pressplay` (private) |
| 2026-09-11 | İlk push | `git push -u origin main` başarılı (SSH). Repo: `github.com/TheBottle2/pressplay` (private). Bundan sonra her işlem sonrası commit+push rutini |
| 2026-09-11 | README İngilizce + install.sh | README tamamen İngilizce'ye çevrildi (yinelenen hotkey maddesi birleştirildi, Quick Start eklendi). Yeni `install.sh`: release derler, `~/.local` altına binary+menü+icon kurar, `input` grubu + udev kuralı (`99-tinytask-uinput`) ile sudo'suz başlatma sağlar. `bash -n` temiz |
| 2026-09-11 | README: lisans bölümü silindi + Wayland netliği | Lisans bölümü kaldırıldı (LICENSE dosyası duruyor). Kullanıcı Wayland'da (`XDG_SESSION_TYPE=wayland` doğrulandı): tablo satırı "çalışmıyor" izlenimi veriyordu → "Wayland'da çalışır, sorun izinlerdir" şeklinde düzeltildi. Not: kullanıcı `input` grubunda değil (sudo şart) — `./install.sh` önerildi |
| 2026-09-11 | README denetimi (7 hata) | En büyüğü: UI Türkçe ama README olmayan İngilizce etiketler gösteriyordu → gerçek etiketler + İngilizce karşılık formatına çevrildi. Diğerleri: yinelenen hotkey maddesi birleştirildi, manuel kurulumda eksik udev kuralı eklendi, doğrulanmamış MSRV (1.70+) → test edilen 1.98, 10k test dürüstlüğü, Türkçe log notu + menü sorun giderme satırı, dosya ağacına LICENSE + güncel fn adı. Commit `b2a9ec8`, push edildi |
| 2026-09-11 | Çok dilli arayüz (9 dil) | Yeni `src/i18n.rs`: en/tr/de/fr/es/pt/it/nl/pl (~54 anahtar), `Lang` enum + `t()` + geri dönüş (önce İngilizce, sonra anahtarın kendisi). Neden sadece Latin: gömülü Ubuntu-Light'ta Kiril yok (font cmap'tan kodla doğrulandı) → ru/zh/ar yol haritasında. `HotkeyConfig.lang` (serde default "tr", eski configler açılır), Ayarlar'da dil seçici (ComboBox, diske kaydedilir), "varsayılana dön" dili korur. `cargo test` 11/11 (4 yeni i18n testi: bütünlük, placeholder, from_code, fallback), warning sıfırlandı, release 13:31 derlendi |
| 2026-09-11 | Varsayılan İngilizce + kalan Türkçe temizliği | `Lang::default` ve config varsayılanı `"en"` oldu. UI dışı ama kullanıcıya görünen tüm metinler İngilizce'ye çevrildi: models dosya-hata mesajları (durum satırında görünüyordu), "keys released" durum mesajı, tüm info/warn/error logları (main/player/recorder/ui) + test mesajları. Kod içi yorumlar Türkçe bırakıldı (geliştiriciye yönelik). `cargo test` 11/11, release 13:40. NOT: kayıtlı config'de `lang:"tr"` varsa dil Türkçe kalır — Ayarlar'dan bir kez değiştirilmeli veya config silinmeli |
| 2026-09-11 | Yazar adı Luna → TheBottle2 | `Cargo.toml` authors + 9 dilde Hakkında/About imzası değiştirildi. `cargo test` 11/11, release yenilendi, push edildi |
| 2026-09-11 | 5'li özellik paketi | (1) Hız çarpanı 0.25x–4x (`SetSpeedMultiplier`, delay + loop-gap ölçeklenir, kelepçeli, testli). (2) Hotkey filtreleme: tetik tuşu + chord modifier'ları kayda alınmaz (basılı-takip + baskılama seti + geriye dönük buffer temizliği, testli). (3) Temiz kapanış: `process::exit` kalktı, Quit yayılımı + join sırası (hotkey→sync→dispatcher→recorder→player), hotkey/sync shutdown bayraklı; headless duman testi UI pencere açtığı için yapılamadı, manuel bekliyor. (4) Son dosyalar: config'de max 8, Makrolar sekmesinde tek-tık yükleme (eksik dosya gri). (5) Kayıt filtresi: klavye/fare onay kutuları (çift taraflı kelepçe). i18n'e 7 anahtar × 9 dil. `cargo test` 16/16, warning yok, release derlendi |
