use crate::models::{AppState, Command, MacroEvent, MacroRecording};
use crossbeam_channel::{Receiver, Sender};
use evdev::{Device, EventType};
use log::{debug, error, info};
use std::collections::HashMap;
use std::fs;
use std::os::unix::io::{AsRawFd, BorrowedFd};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::SystemTime;

pub struct Recorder {
    state: Arc<Mutex<AppState>>,
    recording: Arc<Mutex<MacroRecording>>,
    /// Player'ın okuduğu event kopyası — LoadMacro sonrası senkronizasyon için.
    /// (Sync thread sadece Recording→Idle geçişinde kopyalar, Load'da geçiş olmaz.)
    player_events: Arc<Mutex<Vec<MacroEvent>>>,
    cmd_rx: Receiver<Command>,
    event_tx: Sender<String>,
}

impl Recorder {
    pub fn new(
        state: Arc<Mutex<AppState>>,
        recording: Arc<Mutex<MacroRecording>>,
        player_events: Arc<Mutex<Vec<MacroEvent>>>,
        cmd_rx: Receiver<Command>,
        event_tx: Sender<String>,
    ) -> Self {
        Self {
            state,
            recording,
            player_events,
            cmd_rx,
            event_tx,
        }
    }

    fn enumerate_input_devices() -> Vec<PathBuf> {
        let mut devices = Vec::new();

        let entries = match fs::read_dir("/dev/input") {
            Ok(e) => e,
            Err(_) => {
                error!("Cannot read /dev/input - root or input group required!");
                return devices;
            }
        };

        for entry in entries {
            if let Ok(entry) = entry {
                let path = entry.path();
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    if name.starts_with("event") {
                        if let Ok(device) = Device::open(&path) {
                            let has_key = device.supported_events().contains(EventType::KEY);
                            let has_rel = device.supported_events().contains(EventType::RELATIVE);
                            let has_abs = device.supported_events().contains(EventType::ABSOLUTE);

                            let name_str = device.name().unwrap_or("").to_lowercase();
                            let is_power = name_str.contains("power")
                                || name_str.contains("video")
                                || name_str.contains("lid");

                            if (has_key || has_rel || has_abs) && !is_power {
                                info!(
                                    "Recording device: {:?} - {}",
                                    path,
                                    device.name().unwrap_or("unknown")
                                );
                                devices.push(path);
                            }
                        }
                    }
                }
            }
        }

        devices
    }

    pub fn run(self) {
        let device_paths = Self::enumerate_input_devices();

        if device_paths.is_empty() {
            error!("No input devices found!");
            return;
        }

        let mut devices: HashMap<PathBuf, Device> = HashMap::new();
        for path in &device_paths {
            if let Ok(device) = Device::open(path) {
                devices.insert(path.clone(), device);
            }
        }

        let mut recording_active = false;
        let mut start_time = SystemTime::now();
        let mut local_buffer: Vec<MacroEvent> = Vec::with_capacity(10_000);

        info!("Recorder thread started. Watching {} devices.", devices.len());

        use nix::poll::{poll, PollFd, PollFlags, PollTimeout};

        loop {
            if let Ok(cmd) = self.cmd_rx.try_recv() {
                match cmd {
                    Command::StartRecording => {
                        recording_active = true;
                        start_time = SystemTime::now();
                        local_buffer.clear();
                        *self.state.lock().unwrap() = AppState::Recording;
                        self.event_tx.send("Recording started".to_string()).ok();
                        info!("Recording started");
                    }
                    Command::StopRecording => {
                        recording_active = false;
                        // Local buffer'ı ana recording'e aktar + süreyi hesapla
                        let (count, duration_us) = {
                            let mut rec = self.recording.lock().unwrap();
                            rec.events = local_buffer.clone();
                            rec.duration_us = local_buffer
                                .last()
                                .map(|e| e.timestamp_us)
                                .unwrap_or(0);
                            (rec.events.len(), rec.duration_us)
                        };
                        *self.state.lock().unwrap() = AppState::Idle;
                        self.event_tx
                            .send(format!(
                                "Recording stopped ({} events, {})",
                                count,
                                MacroRecording::format_duration(duration_us)
                            ))
                            .ok();
                        info!(
                            "Recording stopped, {} events recorded (duration: {}µs)",
                            count, duration_us
                        );
                    }
                    Command::SaveMacro(path) => {
                        let result = {
                            let rec = self.recording.lock().unwrap();
                            if rec.events.is_empty() {
                                Err("Kaydedilecek event yok!".to_string())
                            } else {
                                rec.save_to_file(&path)
                            }
                        };
                        match result {
                            Ok(()) => {
                                self.event_tx
                                    .send(format!("Macro saved: {}", path))
                                    .ok();
                                info!("Macro saved: {}", path);
                            }
                            Err(e) => {
                                self.event_tx.send(format!("Save failed: {}", e)).ok();
                                error!("Macro save error: {}", e);
                            }
                        }
                    }
                    Command::LoadMacro(path) => {
                        match MacroRecording::load_from_file(&path) {
                            Ok(loaded) => {
                                let (count, duration_us, name) = (
                                    loaded.events.len(),
                                    loaded.duration_us,
                                    loaded.name.clone(),
                                );
                                {
                                    let mut rec = self.recording.lock().unwrap();
                                    *rec = loaded;
                                }
                                // Player kopyasını da hemen güncelle (sync thread beklemeden)
                                {
                                    let rec = self.recording.lock().unwrap();
                                    let mut pe = self.player_events.lock().unwrap();
                                    *pe = rec.events.clone();
                                }
                                self.event_tx
                                    .send(format!(
                                        "Macro loaded: {} ({} events, {})",
                                        name,
                                        count,
                                        MacroRecording::format_duration(duration_us)
                                    ))
                                    .ok();
                                info!("Macro loaded: {} ({} events)", path, count);
                            }
                            Err(e) => {
                                self.event_tx.send(format!("Load failed: {}", e)).ok();
                                error!("Macro load error: {}", e);
                            }
                        }
                    }
                    Command::Quit => {
                        info!("Recorder thread stopping");
                        break;
                    }
                    _ => {}
                }
            }

            let mut fds: Vec<PollFd> = devices
                .values()
                .map(|dev| unsafe {
                    let borrowed = BorrowedFd::borrow_raw(dev.as_raw_fd());
                    PollFd::new(borrowed, PollFlags::POLLIN)
                })
                .collect();

            // 1ms timeout - düşük gecikme için kritik
            match poll(&mut fds, PollTimeout::from(1u16)) {
                Ok(n) => {
                    if n > 0 {
                        let mut paths_with_events = Vec::new();
                        for (i, fd) in fds.iter().enumerate() {
                            if let Some(revents) = fd.revents() {
                                if revents.contains(PollFlags::POLLIN) {
                                    if let Some((path, _)) = devices.iter().nth(i) {
                                        paths_with_events.push(path.clone());
                                    }
                                }
                            }
                        }

                        for path in paths_with_events {
                            if let Some(device) = devices.get_mut(&path) {
                                if let Ok(events) = device.fetch_events() {
                                    for event in events {
                                        if event.event_type() == EventType::SYNCHRONIZATION {
                                            continue;
                                        }

                                        if recording_active {
                                            let macro_event =
                                                MacroEvent::from_evdev(&event, start_time);
                                            local_buffer.push(macro_event);
                                            debug!("Kaydedildi: {:?}", macro_event);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    error!("Poll error: {}", e);
                }
            }
        }
    }
}