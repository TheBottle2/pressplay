use crate::i18n::{t, Lang};
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
    speed: f32,
    rec_keyboard: bool,
    rec_mouse: bool,
    assigning_hotkey: Option<HotkeyAction>,
    key_capture_active: bool,
    hotkey_msg: String,
    lang: Lang,
}

impl TinyTaskApp {
    pub fn new(
        state: Arc<Mutex<AppState>>,
        recording: Arc<Mutex<MacroRecording>>,
        hotkey_config: Arc<Mutex<HotkeyConfig>>,
        cmd_tx: Sender<Command>,
        event_rx: Receiver<String>,
        stop_flag: Arc<AtomicBool>,
        lang: Lang,
    ) -> Self {
        Self {
            state,
            recording,
            hotkey_config,
            cmd_tx,
            event_rx,
            stop_flag,
            status_message: t(lang, "ready").to_string(),
            event_count: 0,
            current_tab: 0,
            loop_count: 1,
            infinite_loop: false,
            speed: 1.0,
            rec_keyboard: true,
            rec_mouse: true,
            assigning_hotkey: None,
            key_capture_active: false,
            hotkey_msg: String::new(),
            lang,
        }
    }

    fn tr(&self, key: &'static str) -> &'static str {
        t(self.lang, key)
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

            let (tab0, tab1, tab2, tab3) = (
                self.tr("tab_control"),
                self.tr("tab_macros"),
                self.tr("tab_settings"),
                self.tr("tab_about"),
            );
            ui.horizontal(|ui| {
                ui.selectable_value(&mut self.current_tab, 0, tab0);
                ui.selectable_value(&mut self.current_tab, 1, tab1);
                ui.selectable_value(&mut self.current_tab, 2, tab2);
                ui.selectable_value(&mut self.current_tab, 3, tab3);
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
        let (rec_idle, btn_stop, play_idle) =
            (self.tr("rec_idle"), self.tr("btn_stop"), self.tr("play_idle"));
        ui.horizontal(|ui| {
            let (rec_text, rec_enabled, play_enabled) = match current_state {
                AppState::Idle => (rec_idle, true, self.event_count > 0),
                AppState::Recording => (btn_stop, true, false),
                // Playing'de HER İKİ buton da acil stop: kullanıcı hangisine
                // basarsa bassın durmalı (önceden sol buton ölüydü).
                AppState::Playing => (btn_stop, true, true),
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
                btn_stop
            } else {
                play_idle
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

        ui.label(self.tr("loop_title"));
        let (loop_infinite, loop_times, loop_apply) = (
            self.tr("loop_infinite"),
            self.tr("loop_times"),
            self.tr("loop_apply"),
        );
        ui.horizontal(|ui| {
            ui.checkbox(&mut self.infinite_loop, loop_infinite);

            if !self.infinite_loop {
                ui.add(egui::Slider::new(&mut self.loop_count, 1..=9999).text(loop_times));
            }
        });

        if ui.button(loop_apply).clicked() {
            let count = if self.infinite_loop { 0 } else { self.loop_count };
            self.cmd_tx.send(Command::SetLoopCount(count)).ok();
        }

        ui.add_space(5.0);

        // Kayıt filtresi: en az biri açık kalmalı
        let (rec_keyboard, rec_mouse) = (self.tr("rec_keyboard"), self.tr("rec_mouse"));
        let filter_before = (self.rec_keyboard, self.rec_mouse);
        ui.horizontal(|ui| {
            ui.checkbox(&mut self.rec_keyboard, rec_keyboard);
            ui.checkbox(&mut self.rec_mouse, rec_mouse);
        });
        if (self.rec_keyboard, self.rec_mouse) != filter_before {
            if !self.rec_keyboard && !self.rec_mouse {
                self.rec_keyboard = true; // ikisi birden kapanamaz
            }
            self.cmd_tx
                .send(Command::SetRecordFilter {
                    keyboard: self.rec_keyboard,
                    mouse: self.rec_mouse,
                })
                .ok();
        }

        ui.add_space(5.0);

        // Oynatma hızı
        let (speed_label, speed_apply) = (self.tr("speed_label"), self.tr("speed_apply"));
        ui.horizontal(|ui| {
            ui.label(speed_label);
            ui.add(egui::Slider::new(&mut self.speed, 0.25..=4.0).text("x"));
            ui.label(format!("{:.2}x", self.speed));
        });
        if ui.button(speed_apply).clicked() {
            self.cmd_tx.send(Command::SetSpeedMultiplier(self.speed)).ok();
        }

        ui.add_space(10.0);
        ui.separator();

        ui.label(self.tr("file_row"));
        let (save_quick, load_quick) = (self.tr("save_quick"), self.tr("load_quick"));
        ui.horizontal(|ui| {
            if ui.button(save_quick).clicked() {
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
            if ui.button(load_quick).clicked() {
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

        ui.label(self.tr("status_line").replace("{m}", &self.status_message));
        ui.label(
            self.tr("events_line")
                .replace("{n}", &self.event_count.to_string()),
        );
        {
            let rec = self.recording.lock().unwrap();
            ui.label(
                self.tr("macro_line")
                    .replace("{name}", &rec.name)
                    .replace("{dur}", &rec.duration_string())
                    .replace("{ts}", &rec.created_at.to_string()),
            );
        }

        let state_color = match current_state {
            AppState::Idle => egui::Color32::GRAY,
            AppState::Recording => egui::Color32::RED,
            AppState::Playing => egui::Color32::GREEN,
        };

        ui.colored_label(
            state_color,
            match current_state {
                AppState::Idle => self.tr("state_idle"),
                AppState::Recording => self.tr("state_recording"),
                AppState::Playing => self.tr("state_playing"),
            },
        );
    }

    fn show_macros_tab(&mut self, ui: &mut egui::Ui) {
        ui.heading(self.tr("macros_heading"));
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

        ui.label(self.tr("macros_name").replace("{v}", &name));
        ui.label(self.tr("macros_dur").replace("{v}", &duration));
        ui.label(self.tr("macros_events").replace("{v}", &count.to_string()));
        ui.label(
            self.tr("macros_created")
                .replace("{v}", &created_at.to_string()),
        );

        ui.add_space(10.0);
        ui.separator();

        let (macros_save, macros_load) = (self.tr("macros_save"), self.tr("macros_load"));
        ui.horizontal(|ui| {
            if ui.button(macros_save).clicked() {
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
            if ui.button(macros_load).clicked() {
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
        ui.label(self.tr("macros_fmt1"));
        ui.label(self.tr("macros_fmt2"));
        if count == 0 {
            ui.colored_label(egui::Color32::YELLOW, self.tr("macros_empty"));
        }

        ui.add_space(10.0);
        ui.separator();
        ui.label(self.tr("recent_title"));
        {
            let recent = self.hotkey_config.lock().unwrap().recent_files.clone();
            if recent.is_empty() {
                ui.label(self.tr("recent_empty"));
            } else {
                let missing = self.tr("recent_missing");
                for path in &recent {
                    let file_name = std::path::Path::new(path)
                        .file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_else(|| path.clone());
                    let exists = std::path::Path::new(path).exists();
                    let label = if exists {
                        file_name
                    } else {
                        format!("{} {}", file_name, missing)
                    };
                    if ui.add_enabled(exists, egui::Button::new(label)).clicked() {
                        self.cmd_tx.send(Command::LoadMacro(path.clone())).ok();
                    }
                }
            }
        }
    }

    fn show_settings_tab(&mut self, ui: &mut egui::Ui) {
        ui.heading(self.tr("set_heading"));
        ui.add_space(5.0);

        let config = self.hotkey_config.lock().unwrap().clone();
        let set_change = self.tr("set_change");

        ui.horizontal(|ui| {
            ui.label(self.tr("set_record"));
            ui.label(config.record.to_string());

            if ui.button(set_change).clicked() {
                self.assigning_hotkey = Some(HotkeyAction::Record);
                self.key_capture_active = true;
            }
        });

        ui.horizontal(|ui| {
            ui.label(self.tr("set_play"));
            ui.label(config.play.to_string());

            if ui.button(set_change).clicked() {
                self.assigning_hotkey = Some(HotkeyAction::Play);
                self.key_capture_active = true;
            }
        });

        ui.horizontal(|ui| {
            ui.label(self.tr("set_stop"));
            ui.label(config.stop.to_string());

            if ui.button(set_change).clicked() {
                self.assigning_hotkey = Some(HotkeyAction::Stop);
                self.key_capture_active = true;
            }
        });

        ui.add_space(10.0);

        if self.key_capture_active {
            ui.colored_label(egui::Color32::YELLOW, self.tr("set_capture"));
            ui.label(self.tr("set_capture2"));

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
                        self.hotkey_msg =
                            self.tr("set_assigned").replace("{v}", &label);
                    }
                    self.assigning_hotkey = None;
                    self.key_capture_active = false;
                }
            }

            if ui.button(self.tr("set_cancel")).clicked() {
                self.assigning_hotkey = None;
                self.key_capture_active = false;
            }
        }

        if !self.hotkey_msg.is_empty() {
            ui.colored_label(egui::Color32::GREEN, &self.hotkey_msg);
        }

        ui.add_space(5.0);
        ui.label(self.tr("set_warn"));

        ui.add_space(10.0);
        ui.separator();

        // Language picker (persisted to config)
        let set_lang = self.tr("set_lang");
        let lang_before = self.lang;
        ui.horizontal(|ui| {
            ui.label(set_lang);
            egui::ComboBox::from_id_source("lang_combo")
                .selected_text(self.lang.label())
                .show_ui(ui, |ui| {
                    for l in Lang::all() {
                        ui.selectable_value(&mut self.lang, *l, l.label());
                    }
                });
        });
        if self.lang != lang_before {
            let mut cfg = self.hotkey_config.lock().unwrap();
            cfg.lang = self.lang.code().to_string();
            crate::save_config(&cfg);
        }

        ui.add_space(10.0);
        ui.separator();

        if ui.button(self.tr("set_reset")).clicked() {
            // Defaults reset hotkeys but preserve the UI language
            let mut defaults = HotkeyConfig::default();
            defaults.lang = self.lang.code().to_string();
            crate::save_config(&defaults);
            *self.hotkey_config.lock().unwrap() = defaults;
            self.hotkey_msg = self.tr("set_defaults").to_string();
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
        ui.heading(self.tr("about_heading"));
        ui.add_space(10.0);
        ui.label("Linux TinyTask v0.1.0");
        ui.label(self.tr("about_sub"));
        ui.add_space(5.0);
        ui.label(self.tr("about_feat"));
        ui.label(self.tr("about_f1"));
        ui.label(self.tr("about_f2"));
        ui.label(self.tr("about_f3"));
        ui.label(self.tr("about_f4"));
        ui.label(self.tr("about_f5"));
        ui.label(self.tr("about_f6"));
        ui.add_space(10.0);
        ui.label(self.tr("about_credit"));
    }
}

pub fn run_ui(
    state: Arc<Mutex<AppState>>,
    recording: Arc<Mutex<MacroRecording>>,
    hotkey_config: Arc<Mutex<HotkeyConfig>>,
    cmd_tx: Sender<Command>,
    event_rx: Receiver<String>,
    stop_flag: Arc<AtomicBool>,
    lang: Lang,
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
        Box::new(move |_cc| {
            Box::new(TinyTaskApp::new(
                state,
                recording,
                hotkey_config,
                cmd_tx,
                event_rx,
                stop_flag,
                lang,
            )) as Box<dyn eframe::App>
        }),
    ) {
        log::error!("UI error: {:?}", e);
    }
}