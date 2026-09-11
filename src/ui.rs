use crate::models::{AppState, Command, HotkeyAction, HotkeyConfig, KeyCombo, MacroRecording};
use crossbeam_channel::{Receiver, Sender};
use eframe::{egui, NativeOptions};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};

pub struct TinyTaskApp {
    state: Arc<Mutex<AppState>>,
    recording: Arc<Mutex<MacroRecording>>,
    hotkey_config: Arc<Mutex<HotkeyConfig>>,
    cmd_tx: Sender<Command>,
    event_rx: Receiver<String>,
    /// Player ile paylaşılan acil-stop bayrağı: dispatcher/kanal gecikmesinden
    /// bağımsız, tek bir atomic store ile player döngülerini durdurur.
    stop_flag: Arc<AtomicBool>,
    status_message: String,
    event_count: usize,
    current_tab: usize,
    loop_count: u32,
    infinite_loop: bool,
    assigning_hotkey: Option<HotkeyAction>,
    key_capture_active: bool,
    hotkey_msg: String,
}

impl TinyTaskApp {
    pub fn new(
        state: Arc<Mutex<AppState>>,
        recording: Arc<Mutex<MacroRecording>>,
        hotkey_config: Arc<Mutex<HotkeyConfig>>,
        cmd_tx: Sender<Command>,
        event_rx: Receiver<String>,
        stop_flag: Arc<AtomicBool>,
    ) -> Self {
        Self {
            state,
            recording,
            hotkey_config,
            cmd_tx,
            event_rx,
            stop_flag,
            status_message: "Hazır".to_string(),
            event_count: 0,
            current_tab: 0,
            loop_count: 1,
            infinite_loop: false,
            assigning_hotkey: None,
            key_capture_active: false,
            hotkey_msg: String::new(),
        }
    }

    /// Acil durdurma: önce paylaşılan bayrağı set et (player anında görür),
    /// sonra normal komutu da gönder (durum Idle'a geçsin, log düşsün).
    fn request_stop(&self) {
        self.stop_flag.store(true, Ordering::Relaxed);
        self.cmd_tx.send(Command::StopPlayback).ok();
    }

    fn update_status(&mut self) {
        if let Ok(msg) = self.event_rx.try_recv() {
            self.status_message = msg;
        }

        let rec = self.recording.lock().unwrap();
        self.event_count = rec.events.len();
    }
}

impl eframe::App for TinyTaskApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.update_status();

        egui::CentralPanel::default().show(ctx, |ui| {
            let current_state = *self.state.lock().unwrap();

            ui.heading("Linux TinyTask");
            ui.separator();

            ui.horizontal(|ui| {
                ui.selectable_value(&mut self.current_tab, 0, "Kontrol");
                ui.selectable_value(&mut self.current_tab, 1, "Makrolar");
                ui.selectable_value(&mut self.current_tab, 2, "Ayarlar");
                ui.selectable_value(&mut self.current_tab, 3, "Hakkında");
            });

            ui.separator();

            match self.current_tab {
                0 => self.show_control_tab(ui, current_state),
                1 => self.show_macros_tab(ui),
                2 => self.show_settings_tab(ui),
                3 => self.show_about_tab(ui),
                _ => {}
            }
        });

        ctx.request_repaint_after(std::time::Duration::from_millis(50));
    }
}

impl TinyTaskApp {
    fn show_control_tab(&mut self, ui: &mut egui::Ui, current_state: AppState) {
        ui.horizontal(|ui| {
            let (rec_text, rec_enabled, play_enabled) = match current_state {
                AppState::Idle => ("● Kaydet", true, self.event_count > 0),
                AppState::Recording => ("■ Durdur", true, false),
                // Playing'de HER İKİ buton da acil stop: kullanıcı hangisine
                // basarsa bassın durmalı (önceden sol buton ölüydü).
                AppState::Playing => ("■ Durdur", true, true),
            };

            if ui
                .add_enabled(
                    rec_enabled,
                    egui::Button::new(rec_text)
                        .fill(if current_state == AppState::Idle {
                            egui::Color32::from_rgb(180, 0, 0)
                        } else {
                            egui::Color32::DARK_RED
                        })
                        .min_size(egui::vec2(100.0, 40.0)),
                )
                .clicked()
            {
                match current_state {
                    AppState::Idle => {
                        self.cmd_tx.send(Command::StartRecording).ok();
                    }
                    AppState::Recording => {
                        self.cmd_tx.send(Command::StopRecording).ok();
                    }
                    AppState::Playing => {
                        // Soldaki buton da acil stop görevi görür
                        self.request_stop();
                    }
                }
            }

            let play_text = if current_state == AppState::Playing {
                "■ Durdur"
            } else {
                "▶ Oynat"
            };

            if ui
                .add_enabled(
                    play_enabled,
                    egui::Button::new(play_text)
                        .fill(if current_state == AppState::Playing {
                            egui::Color32::DARK_RED
                        } else {
                            egui::Color32::from_rgb(0, 120, 0)
                        })
                        .min_size(egui::vec2(100.0, 40.0)),
                )
                .clicked()
            {
                if current_state == AppState::Idle {
                    self.cmd_tx.send(Command::StartPlayback).ok();
                } else {
                    self.request_stop();
                }
            }
        });

        ui.add_space(10.0);
        ui.separator();

        ui.label("Döngü Ayarları:");
        ui.horizontal(|ui| {
            ui.checkbox(&mut self.infinite_loop, "Sonsuz döngü");

            if !self.infinite_loop {
                ui.add(egui::Slider::new(&mut self.loop_count, 1..=9999).text("kez"));
            }
        });

        if ui.button("Döngü Ayarını Uygula").clicked() {
            let count = if self.infinite_loop { 0 } else { self.loop_count };
            self.cmd_tx.send(Command::SetLoopCount(count)).ok();
        }

        ui.add_space(10.0);
        ui.separator();

        ui.label("Dosya:");
        ui.horizontal(|ui| {
            if ui.button("💾 Kaydet").clicked() {
                let name = self.recording.lock().unwrap().name.clone();
                if let Some(path) = rfd::FileDialog::new()
                    .add_filter("TinyTask Macro", &["tts"])
                    .add_filter("JSON", &["json"])
                    .set_file_name(format!("{}.tts", name))
                    .save_file()
                {
                    self.cmd_tx
                        .send(Command::SaveMacro(path.to_string_lossy().to_string()))
                        .ok();
                }
            }
            if ui.button("📂 Yükle").clicked() {
                if let Some(path) = rfd::FileDialog::new()
                    .add_filter("TinyTask Macro", &["tts", "json"])
                    .pick_file()
                {
                    self.cmd_tx
                        .send(Command::LoadMacro(path.to_string_lossy().to_string()))
                        .ok();
                }
            }
        });

        ui.add_space(10.0);
        ui.separator();

        ui.label(format!("Durum: {}", self.status_message));
        ui.label(format!("Kaydedilen event: {}", self.event_count));
        {
            let rec = self.recording.lock().unwrap();
            ui.label(format!(
                "Makro: {} | Süre: {} | Oluşturulma: {}",
                rec.name,
                rec.duration_string(),
                rec.created_at
            ));
        }

        let state_color = match current_state {
            AppState::Idle => egui::Color32::GRAY,
            AppState::Recording => egui::Color32::RED,
            AppState::Playing => egui::Color32::GREEN,
        };

        ui.colored_label(
            state_color,
            match current_state {
                AppState::Idle => "○ Boşta",
                AppState::Recording => "● KAYIT YAPILIYOR",
                AppState::Playing => "● OYNATILIYOR",
            },
        );
    }

    fn show_macros_tab(&mut self, ui: &mut egui::Ui) {
        ui.heading("Makro Yönetimi");
        ui.add_space(5.0);

        let (name, created_at, duration, count) = {
            let rec = self.recording.lock().unwrap();
            (
                rec.name.clone(),
                rec.created_at,
                rec.duration_string(),
                rec.events.len(),
            )
        };

        ui.label(format!("Ad: {}", name));
        ui.label(format!("Süre: {}", duration));
        ui.label(format!("Event sayısı: {}", count));
        ui.label(format!("Oluşturulma (unix): {}", created_at));

        ui.add_space(10.0);
        ui.separator();

        ui.horizontal(|ui| {
            if ui.button("💾 Makroyu Kaydet").clicked() {
                // Native dialog; UI thread'de açılır, IO recorder thread'de yapılır.
                if let Some(path) = rfd::FileDialog::new()
                    .add_filter("TinyTask Macro", &["tts"])
                    .add_filter("JSON", &["json"])
                    .set_file_name(format!("{}.tts", name))
                    .save_file()
                {
                    let p = path.to_string_lossy().to_string();
                    self.cmd_tx.send(Command::SaveMacro(p)).ok();
                }
            }
            if ui.button("📂 Makro Yükle").clicked() {
                if let Some(path) = rfd::FileDialog::new()
                    .add_filter("TinyTask Macro", &["tts", "json"])
                    .pick_file()
                {
                    let p = path.to_string_lossy().to_string();
                    self.cmd_tx.send(Command::LoadMacro(p)).ok();
                }
            }
        });

        ui.add_space(5.0);
        ui.label("Format: .tts (binary, küçük/hızlı) veya .json (okunabilir/debug).");
        ui.label("Yüklenen makro anında oynatmaya hazır hale gelir.");
        if count == 0 {
            ui.colored_label(
                egui::Color32::YELLOW,
                "Henüz kayıt yok — önce Kontrol sekmesinde kayıt yapın veya dosya yükleyin.",
            );
        }
    }

    fn show_settings_tab(&mut self, ui: &mut egui::Ui) {
        ui.heading("Klavye Kısayolları");
        ui.add_space(5.0);

        let config = self.hotkey_config.lock().unwrap().clone();

        ui.horizontal(|ui| {
            ui.label("Kayıt Başlat/Durdur:");
            ui.label(config.record.to_string());

            if ui.button("Değiştir").clicked() {
                self.assigning_hotkey = Some(HotkeyAction::Record);
                self.key_capture_active = true;
            }
        });

        ui.horizontal(|ui| {
            ui.label("Oynat Başlat:");
            ui.label(config.play.to_string());

            if ui.button("Değiştir").clicked() {
                self.assigning_hotkey = Some(HotkeyAction::Play);
                self.key_capture_active = true;
            }
        });

        ui.horizontal(|ui| {
            ui.label("Oynatmayı Durdur:");
            ui.label(config.stop.to_string());

            if ui.button("Değiştir").clicked() {
                self.assigning_hotkey = Some(HotkeyAction::Stop);
                self.key_capture_active = true;
            }
        });

        ui.add_space(10.0);

        if self.key_capture_active {
            ui.colored_label(
                egui::Color32::YELLOW,
                "Yeni kısayolu girin: tek tuş (örn. F8) veya Ctrl/Alt/Shift ile birlikte bir tuş...",
            );
            ui.label("Atamak için tuşa basın. Vazgeçmek için İptal'e tıklayın.");

            // Bu frame'de basılan (edge) ilk tuşu yakala
            let mut captured: Option<(egui::Key, egui::Modifiers)> = None;
            ui.ctx().input(|i| {
                for k in Self::CAPTURE_KEYS {
                    if i.key_pressed(*k) {
                        captured = Some((*k, i.modifiers));
                        break;
                    }
                }
            });
            if let Some((key, mods)) = captured {
                if let Some(code) = Self::egui_key_to_evdev(key) {
                    let combo = KeyCombo {
                        ctrl: mods.ctrl,
                        alt: mods.alt,
                        shift: mods.shift,
                        super_key: false, // egui Super tuşunu bildirmez
                        key_code: code,
                    };
                    let label = combo.to_string();
                    if let Some(action) = self.assigning_hotkey {
                        {
                            let mut cfg = self.hotkey_config.lock().unwrap();
                            match action {
                                HotkeyAction::Record => cfg.record = combo.clone(),
                                HotkeyAction::Play => cfg.play = combo.clone(),
                                HotkeyAction::Stop => cfg.stop = combo.clone(),
                            }
                            // Kalıcı kaydet (hotkey thread aynı Arc'i okur, anında aktif)
                            crate::save_config(&cfg);
                        }
                        self.cmd_tx.send(Command::SetHotkey(action, combo)).ok();
                        self.hotkey_msg = format!("Atandı: {}", label);
                    }
                    self.assigning_hotkey = None;
                    self.key_capture_active = false;
                }
            }

            if ui.button("İptal").clicked() {
                self.assigning_hotkey = None;
                self.key_capture_active = false;
            }
        }

        if !self.hotkey_msg.is_empty() {
            ui.colored_label(egui::Color32::GREEN, &self.hotkey_msg);
        }

        ui.add_space(5.0);
        ui.label("⚠ Tek tuş (örn. A, Space) yazı yazarken de tetiklenir. F8–F12 önerilir.");

        ui.add_space(10.0);
        ui.separator();

        if ui.button("Varsayılan Kısayollara Döndür").clicked() {
            let defaults = HotkeyConfig::default();
            crate::save_config(&defaults);
            *self.hotkey_config.lock().unwrap() = defaults;
            self.hotkey_msg = "Varsayılanlara dönüldü.".to_string();
        }
    }

    /// Yakalanabilir tuş listesi (harf + rakam + F1-F12 + özel tuşlar)
    const CAPTURE_KEYS: &'static [egui::Key] = &[
        egui::Key::A, egui::Key::B, egui::Key::C, egui::Key::D, egui::Key::E,
        egui::Key::F, egui::Key::G, egui::Key::H, egui::Key::I, egui::Key::J,
        egui::Key::K, egui::Key::L, egui::Key::M, egui::Key::N, egui::Key::O,
        egui::Key::P, egui::Key::Q, egui::Key::R, egui::Key::S, egui::Key::T,
        egui::Key::U, egui::Key::V, egui::Key::W, egui::Key::X, egui::Key::Y,
        egui::Key::Z, egui::Key::Num0, egui::Key::Num1, egui::Key::Num2,
        egui::Key::Num3, egui::Key::Num4, egui::Key::Num5, egui::Key::Num6,
        egui::Key::Num7, egui::Key::Num8, egui::Key::Num9, egui::Key::F1,
        egui::Key::F2, egui::Key::F3, egui::Key::F4, egui::Key::F5, egui::Key::F6,
        egui::Key::F7, egui::Key::F8, egui::Key::F9, egui::Key::F10,
        egui::Key::F11, egui::Key::F12, egui::Key::Escape, egui::Key::Tab,
        egui::Key::Backspace, egui::Key::Enter, egui::Key::Space,
        egui::Key::Insert, egui::Key::Delete, egui::Key::Home, egui::Key::End,
        egui::Key::PageUp, egui::Key::PageDown, egui::Key::ArrowUp,
        egui::Key::ArrowDown, egui::Key::ArrowLeft, egui::Key::ArrowRight,
    ];

    /// egui tuşu -> Linux evdev koduna çevirir
    fn egui_key_to_evdev(key: egui::Key) -> Option<u16> {
        use egui::Key as K;
        let code = match key {
            K::A => 30, K::B => 48, K::C => 46, K::D => 32, K::E => 18,
            K::F => 33, K::G => 34, K::H => 35, K::I => 23, K::J => 36,
            K::K => 37, K::L => 38, K::M => 50, K::N => 49, K::O => 24,
            K::P => 25, K::Q => 16, K::R => 19, K::S => 31, K::T => 20,
            K::U => 22, K::V => 47, K::W => 17, K::X => 45, K::Y => 21,
            K::Z => 44,
            K::Num1 => 2, K::Num2 => 3, K::Num3 => 4, K::Num4 => 5,
            K::Num5 => 6, K::Num6 => 7, K::Num7 => 8, K::Num8 => 9,
            K::Num9 => 10, K::Num0 => 11,
            K::F1 => 59, K::F2 => 60, K::F3 => 61, K::F4 => 62,
            K::F5 => 63, K::F6 => 64, K::F7 => 65, K::F8 => 66,
            K::F9 => 67, K::F10 => 68, K::F11 => 87, K::F12 => 88,
            K::Escape => 1, K::Tab => 15, K::Backspace => 14,
            K::Enter => 28, K::Space => 57,
            K::Insert => 110, K::Delete => 111, K::Home => 102,
            K::End => 107, K::PageUp => 104, K::PageDown => 109,
            K::ArrowUp => 103, K::ArrowDown => 108,
            K::ArrowLeft => 105, K::ArrowRight => 106,
            _ => return None,
        };
        Some(code)
    }

    fn show_about_tab(&mut self, ui: &mut egui::Ui) {
        ui.heading("Hakkında");
        ui.add_space(10.0);
        ui.label("Linux TinyTask v0.1.0");
        ui.label("Minimalist makro kaydedici ve oynatıcı");
        ui.add_space(5.0);
        ui.label("Özellikler:");
        ui.label("• Kernel seviyesinde girdi yakalama");
        ui.label("• X11/Wayland bağımsız");
        ui.label("• Milisaniye hassasiyeti");
        ui.label("• Oyun desteği");
        ui.label("• Döngü modu");
        ui.label("• Özelleştirilebilir kısayollar");
        ui.add_space(10.0);
        ui.label("Luna tarafından geliştirilmiştir.");
    }
}

pub fn run_ui(
    state: Arc<Mutex<AppState>>,
    recording: Arc<Mutex<MacroRecording>>,
    hotkey_config: Arc<Mutex<HotkeyConfig>>,
    cmd_tx: Sender<Command>,
    event_rx: Receiver<String>,
    stop_flag: Arc<AtomicBool>,
) {
    let options = NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([420.0, 480.0])
            .with_min_inner_size([350.0, 350.0])
            .with_always_on_top()
            .with_decorations(true)
            .with_resizable(true),
        ..Default::default()
    };

    if let Err(e) = eframe::run_native(
        "Linux TinyTask",
        options,
        Box::new(|_cc| {
            Box::new(TinyTaskApp::new(
                state,
                recording,
                hotkey_config,
                cmd_tx,
                event_rx,
                stop_flag,
            )) as Box<dyn eframe::App>
        }),
    ) {
        log::error!("UI hatası: {:?}", e);
    }
}