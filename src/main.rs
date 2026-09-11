mod i18n;
mod models;
mod player;
mod recorder;
mod ui;

use crossbeam_channel::bounded;
use i18n::Lang;
use log::{info, warn};
use models::{AppState, Command, HotkeyConfig, MacroEvent, MacroRecording, RecordFilter};
use player::Player;
use recorder::Recorder;
use std::fs;
use std::path::PathBuf;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};

const CONFIG_FILE: &str = "tinytask_config.json";

pub(crate) fn get_config_path() -> PathBuf {
    let base = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
    let dir = base.join("pressplay");
    if !dir.exists() {
        // Pre-0.2.0 kurulumundan ayarları taşı (hotkey + dil + son dosyalar korunur)
        let old_dir = base.join("linux-tinytask");
        if old_dir.exists() {
            if fs::rename(&old_dir, &dir).is_ok() {
                log::info!("Migrated config: linux-tinytask -> pressplay");
            }
        }
    }
    fs::create_dir_all(&dir).ok();
    dir.join(CONFIG_FILE)
}

fn load_config() -> HotkeyConfig {
    let path = get_config_path();
    if path.exists() {
        if let Ok(data) = fs::read_to_string(&path) {
            if let Ok(config) = serde_json::from_str(&data) {
                return config;
            }
        }
    }
    HotkeyConfig::default()
}

pub(crate) fn save_config(config: &HotkeyConfig) {
    let path = get_config_path();
    if let Ok(data) = serde_json::to_string_pretty(config) {
        fs::write(&path, data).ok();
    }
}

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    info!("Starting PressPlay...");
    info!("NOTE: This app requires /dev/input and /dev/uinput access.");

    let state = Arc::new(Mutex::new(AppState::Idle));
    let recording = Arc::new(Mutex::new(MacroRecording::new("untitled".to_string())));
    let hotkey_config = Arc::new(Mutex::new(load_config()));
    let recording_for_player: Arc<Mutex<Vec<MacroEvent>>> = Arc::new(Mutex::new(Vec::new()));
    // UI <-> Player arası paylaşılan acil-stop bayrağı (kanaldan bağımsız)
    let stop_flag = Arc::new(AtomicBool::new(false));
    // Kapanış bayrağı: sync + hotkey thread'leri bunu yoklar
    let shutdown = Arc::new(AtomicBool::new(false));
    // Klavye/fare kayıt filtresi (recorder'la paylaşılır)
    let record_filter = Arc::new(Mutex::new(RecordFilter::default()));

    // UI -> Dispatcher channel (tek channel, dispatcher yönlendirir)
    let (cmd_tx, cmd_rx_dispatcher) = bounded::<Command>(50);

    // Dispatcher -> Recorder/Player channels (Save/Load path'leri uzun olabilir, buffer geniş)
    let (cmd_tx_recorder, cmd_rx_recorder) = bounded::<Command>(50);
    let (cmd_tx_player, cmd_rx_player) = bounded::<Command>(50);
    // Kapanışta Quit göndermek için yedek tutulur (asıllar dispatcher'a taşınır)
    let quit_recorder = cmd_tx_recorder.clone();
    let quit_player = cmd_tx_player.clone();

    let (event_tx, event_rx) = bounded::<String>(50);

    // Dispatcher thread: komutları doğru thread'lere yönlendirir.
    // try_send kullanılır: hedef kanal dolu olsa bile dispatcher kilitlenmez,
    // acil stop komutları gecikmez (stop bayrağı ayrıca anında iletilir).
    let dispatcher_handle = std::thread::spawn(move || {
        info!("Dispatcher thread started");
        for cmd in cmd_rx_dispatcher {
            match &cmd {
                Command::StartRecording
                | Command::StopRecording
                | Command::SaveMacro(_)
                | Command::LoadMacro(_)
                | Command::SetRecordFilter { .. } => {
                    if cmd_tx_recorder.try_send(cmd).is_err() {
                        warn!("Recorder channel full, command dropped");
                    }
                }
                Command::StartPlayback
                | Command::StopPlayback
                | Command::SetLoopCount(_)
                | Command::SetSpeedMultiplier(_) => {
                    if cmd_tx_player.try_send(cmd).is_err() {
                        warn!("Player channel full, command dropped");
                    }
                }
                Command::SetHotkey(_, _) | Command::SaveConfig => {
                    // Şimdilik no-op, gelecekte config thread'e gönderilecek
                }
                Command::Quit => {
                    cmd_tx_recorder.try_send(Command::Quit).ok();
                    cmd_tx_player.try_send(Command::Quit).ok();
                    break;
                }
            }
        }
        info!("Dispatcher thread stopped");
    });

    // Recorder thread
    let recorder_state = state.clone();
    let recorder_recording = recording.clone();
    let recorder_player_events = recording_for_player.clone();
    let recorder_hotkeys = hotkey_config.clone();
    let recorder_filter = record_filter.clone();
    let event_tx_rec = event_tx.clone();
    let recorder_handle = std::thread::spawn(move || {
        let recorder = Recorder::new(
            recorder_state,
            recorder_recording,
            recorder_player_events,
            recorder_hotkeys,
            recorder_filter,
            cmd_rx_recorder,
            event_tx_rec,
        );
        recorder.run();
    });

    // Player thread
    let player_state = state.clone();
    let event_tx_play = event_tx.clone();
    let recording_for_player_for_closure = recording_for_player.clone();
    let player_stop_flag = stop_flag.clone();
    let player_handle = std::thread::spawn(move || {
        let player = Player::new(
            player_state,
            recording_for_player_for_closure,
            cmd_rx_player,
            event_tx_play,
            player_stop_flag,
        );
        player.run();
    });

    // Recording -> Player senkronizasyon (doğrudan, gecikmesiz)
    let sync_state = state.clone();
    let sync_recording = recording.clone();
    let sync_player_events = recording_for_player.clone();
    let sync_shutdown = shutdown.clone();
    let sync_handle = std::thread::spawn(move || {
        let mut last_state = AppState::Idle;
        while !sync_shutdown.load(Ordering::Relaxed) {
            let current_state = *sync_state.lock().unwrap();

            if last_state == AppState::Recording && current_state == AppState::Idle {
                let rec = sync_recording.lock().unwrap();
                let mut player_events = sync_player_events.lock().unwrap();
                *player_events = rec.events.clone();
                info!("{} events synced to player", player_events.len());
            }

            last_state = current_state;
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
        info!("Sync thread stopped");
    });

    // Hotkey thread (stop hotkey'i bayrağı da set eder: kanal gecikmesiz acil stop)
    let cmd_tx_hotkey = cmd_tx.clone();
    let hotkey_state = state.clone();
    let hotkey_config_clone = hotkey_config.clone();
    let hotkey_stop_flag = stop_flag.clone();
    let hotkey_shutdown = shutdown.clone();
    let hotkey_handle = std::thread::spawn(move || {
        run_hotkey_thread(
            hotkey_state,
            cmd_tx_hotkey,
            hotkey_config_clone,
            hotkey_stop_flag,
            hotkey_shutdown,
        );
    });

    info!("Starting UI...");
    let lang = Lang::from_code(&hotkey_config.lock().unwrap().lang).unwrap_or_default();
    let cmd_tx_ui = cmd_tx.clone();
    ui::run_ui(state, recording, hotkey_config, cmd_tx_ui, event_rx, stop_flag, lang);

    // Temiz kapanış: pencere kapandı -> önce poll-thread'leri durdur
    // (kanalları tutan tek klonlar onlarda), sonra Quit yay + birleş.
    info!("Shutting down...");
    shutdown.store(true, Ordering::Relaxed);
    hotkey_handle.join().ok();
    sync_handle.join().ok();
    // cmd_tx (UI/hotkey klonları) artık düşmüş olmalı -> dispatcher kanalı kapanır
    drop(cmd_tx);
    dispatcher_handle.join().ok();
    // Recorder/Player'a doğrudan Quit (dispatcher zaten kapalı olabilir)
    quit_recorder.try_send(Command::Quit).ok();
    quit_player.try_send(Command::Quit).ok();
    recorder_handle.join().ok();
    player_handle.join().ok();
    info!("All threads stopped. Bye!");
}

fn run_hotkey_thread(
    state: Arc<Mutex<AppState>>,
    cmd_tx: crossbeam_channel::Sender<Command>,
    hotkey_config: Arc<Mutex<HotkeyConfig>>,
    stop_flag: Arc<AtomicBool>,
    shutdown: Arc<AtomicBool>,
) {
    use evdev::{Device, EventType};
    use nix::poll::{poll, PollFd, PollFlags, PollTimeout};
    use std::collections::HashSet;
    use std::fs;
    use std::os::unix::io::{AsRawFd, BorrowedFd};

    info!("Hotkey thread started");

    let mut devices: Vec<Device> = Vec::new();
    if let Ok(entries) = fs::read_dir("/dev/input") {
        for entry in entries.flatten() {
            let path = entry.path();
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if name.starts_with("event") {
                    if let Ok(device) = Device::open(&path) {
                        if device.supported_events().contains(EventType::KEY) {
                            let name_str = device.name().unwrap_or("").to_lowercase();
                            if !name_str.contains("power") && !name_str.contains("video") {
                                devices.push(device);
                            }
                        }
                    }
                }
            }
        }
    }

    let mut pressed_keys: HashSet<u16> = HashSet::new();
    let mut last_trigger_time = std::time::Instant::now();

    while !shutdown.load(Ordering::Relaxed) {
        if devices.is_empty() {
            // Cihaz yoksa poll boş fd ile busy-loop'a girer; sakin bekle
            std::thread::sleep(std::time::Duration::from_millis(100));
            continue;
        }
        let mut fds: Vec<PollFd> = devices
            .iter()
            .map(|dev| unsafe {
                let borrowed = BorrowedFd::borrow_raw(dev.as_raw_fd());
                PollFd::new(borrowed, PollFlags::POLLIN)
            })
            .collect();

        // 1ms timeout - düşük gecikme
        if let Ok(n) = poll(&mut fds, PollTimeout::from(1u16)) {
            if n > 0 {
                for (i, fd) in fds.iter().enumerate() {
                    if let Some(revents) = fd.revents() {
                        if revents.contains(PollFlags::POLLIN) {
                            if let Some(device) = devices.get_mut(i) {
                                if let Ok(events) = device.fetch_events() {
                                    for event in events {
                                        if event.event_type() == EventType::KEY {
                                            let key_code = event.code();
                                            let value = event.value();

                                            if value == 1 {
                                                pressed_keys.insert(key_code);
                                            } else if value == 0 {
                                                pressed_keys.remove(&key_code);
                                            }

                                            if value == 1
                                                && last_trigger_time.elapsed().as_millis() > 200
                                            {
                                                let config = hotkey_config.lock().unwrap();

                                                if config.record.matches(&pressed_keys) {
                                                    let current_state = *state.lock().unwrap();
                                                    if current_state == AppState::Idle {
                                                        cmd_tx.send(Command::StartRecording).ok();
                                                    } else if current_state == AppState::Recording {
                                                        cmd_tx.send(Command::StopRecording).ok();
                                                    }
                                                    last_trigger_time = std::time::Instant::now();
                                                } else if config.play.matches(&pressed_keys) {
                                                    let current_state = *state.lock().unwrap();
                                                    if current_state == AppState::Idle {
                                                        cmd_tx.send(Command::StartPlayback).ok();
                                                    }
                                                    last_trigger_time = std::time::Instant::now();
                                                } else if config.stop.matches(&pressed_keys) {
                                                    let current_state = *state.lock().unwrap();
                                                    if current_state == AppState::Playing {
                                                        // Bayrak önce (anında), komut sonra (durum/log için)
                                                        stop_flag.store(true, Ordering::Relaxed);
                                                        cmd_tx.send(Command::StopPlayback).ok();
                                                    }
                                                    last_trigger_time = std::time::Instant::now();
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    info!("Hotkey thread stopped");
}