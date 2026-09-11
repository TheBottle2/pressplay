use crate::models::{AppState, Command, MacroEvent};
use crossbeam_channel::{Receiver, Sender};
use evdev::{
    uinput::{VirtualDevice, VirtualDeviceBuilder},
    AttributeSet, EventType, InputEvent, Key, RelativeAxisType,
};
use log::{error, info, warn};
use std::collections::HashSet;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};
use std::thread;
use std::time::{Duration, Instant};

pub struct Player {
    state: Arc<Mutex<AppState>>,
    recording_events: Arc<Mutex<Vec<MacroEvent>>>,
    cmd_rx: Receiver<Command>,
    event_tx: Sender<String>,
    loop_count: Arc<Mutex<u32>>,
    /// UI ile paylaşılan acil-stop bayrağı (kanaldan bağımsız, anında).
    stop_flag: Arc<AtomicBool>,
}

impl Player {
    pub fn new(
        state: Arc<Mutex<AppState>>,
        recording_events: Arc<Mutex<Vec<MacroEvent>>>,
        cmd_rx: Receiver<Command>,
        event_tx: Sender<String>,
        stop_flag: Arc<AtomicBool>,
    ) -> Self {
        Self {
            state,
            recording_events,
            cmd_rx,
            event_tx,
            loop_count: Arc::new(Mutex::new(1)),
            stop_flag,
        }
    }

    fn create_virtual_device() -> std::io::Result<VirtualDevice> {
        let mut keys = AttributeSet::<Key>::new();
        for key_code in 0..768u16 {
            keys.insert(Key::new(key_code));
        }

        let mut rel_axes = AttributeSet::<RelativeAxisType>::new();
        rel_axes.insert(RelativeAxisType::REL_X);
        rel_axes.insert(RelativeAxisType::REL_Y);
        rel_axes.insert(RelativeAxisType::REL_WHEEL);
        rel_axes.insert(RelativeAxisType::REL_HWHEEL);

        VirtualDeviceBuilder::new()?
            .name("LinuxTinyTask Virtual Device")
            .with_keys(&keys)?
            .with_relative_axes(&rel_axes)?
            .build()
    }

    /// Kesilebilir hassas uyku: deadline tabanlı, her ~1ms'de stop flag kontrol eder.
    /// Dönüş: true = süre doldu, false = durduruldu.
    /// Önceki bug: %90 sleep + tam sürenin busy-wait'i = ~1.9x yavaşlama. Düzeltildi.
    fn precise_sleep_interruptible(duration: Duration, stop: &AtomicBool) -> bool {
        if duration.is_zero() {
            return !stop.load(Ordering::Relaxed);
        }
        let deadline = Instant::now() + duration;
        let nanos = duration.as_nanos();

        if nanos < 100_000 {
            // 100µs altı: doğrudan busy-wait (flag kontrollü)
            while Instant::now() < deadline {
                if stop.load(Ordering::Relaxed) {
                    return false;
                }
                std::hint::spin_loop();
            }
            return !stop.load(Ordering::Relaxed);
        }

        // Uzun süreler: önce kaba sleep (deadline - 300µs marj), sonra spin.
        // Sleep'i 1ms parçalara böl ki stop anında gelsin.
        loop {
            if stop.load(Ordering::Relaxed) {
                return false;
            }
            let now = Instant::now();
            if now >= deadline {
                return true;
            }
            let remaining = deadline - now;
            if remaining.as_micros() > 800 {
                // Kalan sürenin tamamını tek sleep'te uyuma; max 2ms parçalar
                let chunk = remaining.min(Duration::from_millis(2));
                thread::sleep(chunk);
            } else {
                // Son ~800µs: busy-wait
                while Instant::now() < deadline {
                    if stop.load(Ordering::Relaxed) {
                        return false;
                    }
                    std::hint::spin_loop();
                }
                return !stop.load(Ordering::Relaxed);
            }
        }
    }

    /// Oynatılan event'ten basılı-tuş takibi: EV_KEY + basıldı(1/2) -> ekle, bırakıldı(0) -> çıkar.
    /// Saf fonksiyon (test edilebilir); mouse butonları da EV_KEY'dir (BTN_*), onlar da kapsanır.
    fn track_held(held: &mut HashSet<u16>, event: &MacroEvent) {
        if event.event_type == EventType::KEY.0 {
            if event.value == 1 || event.value == 2 {
                held.insert(event.code);
            } else if event.value == 0 {
                held.remove(&event.code);
            }
        }
    }

    /// Takılı kalmış tuşları bırakır: stop/finish anında key-up gönderir.
    /// Oynatma key-down'da kesildiyse (veya kayıt tuş basılıyken bittiyse) kernel
    /// tuşu basılı sanmaya devam eder -> klavye/fare "bozuk" davranır. Bunu önler.
    /// Dönüş: bırakılan tuş sayısı.
    fn release_held(device: &mut VirtualDevice, held: &mut HashSet<u16>) -> usize {
        if held.is_empty() {
            return 0;
        }
        let count = held.len();
        for code in held.drain() {
            let ev = InputEvent::new(EventType::KEY, code, 0);
            if let Err(e) = device.emit(&[ev]) {
                error!("Takılı tuş bırakılamadı (code {}): {:?}", code, e);
            }
        }
        info!("{} takılı tuş bırakıldı", count);
        count
    }

    /// Komut kanalını yoklar; Stop/Quit/SetLoopCount'u işler.
    fn poll_commands(
        cmd_rx: &Receiver<Command>,
        state: &Mutex<AppState>,
        event_tx: &Sender<String>,
        loop_count: &Mutex<u32>,
        stop: &AtomicBool,
    ) -> (bool, bool) {
        let mut quit = false;
        let mut stopped = false;
        while let Ok(cmd) = cmd_rx.try_recv() {
            match cmd {
                Command::StopPlayback => {
                    stop.store(true, Ordering::Relaxed);
                    *state.lock().unwrap() = AppState::Idle;
                    event_tx.send("Playback stopped".to_string()).ok();
                    info!("Oynatma durdu (acil stop)");
                    stopped = true;
                }
                Command::SetLoopCount(count) => {
                    *loop_count.lock().unwrap() = count;
                    info!("Loop count ayarlandı: {}", count);
                }
                Command::Quit => {
                    stop.store(true, Ordering::Relaxed);
                    quit = true;
                }
                Command::StartPlayback => {
                    // Oynatma sırasında gelen tekrar başlatma isteğini yoksay
                }
                _ => {}
            }
        }
        (quit, stopped)
    }

    pub fn run(self) {
        let virtual_device = match Self::create_virtual_device() {
            Ok(dev) => {
                info!("Sanal input cihazı oluşturuldu (uinput)");
                dev
            }
            Err(e) => {
                error!(
                    "Sanal cihaz oluşturulamadı: {:?} - root/uinput grubu gerekli!",
                    e
                );
                return;
            }
        };

        let mut device = virtual_device;
        // UI ile paylaşılan acil-stop bayrağı: UI butonu set eder etmez
        // oynatma döngüleri + sleep görür (kanal/dispatcher gecikmesiz).
        let stop_flag = self.stop_flag.clone();
        let mut playing = false;

        info!("Player thread başladı");

        loop {
            // --- komut yoklama (oynatma yokken bloklamayan) ---
            if let Ok(cmd) = self.cmd_rx.try_recv() {
                match cmd {
                    Command::StartPlayback => {
                        playing = true;
                        stop_flag.store(false, Ordering::Relaxed);
                        *self.state.lock().unwrap() = AppState::Playing;
                        self.event_tx.send("Playback started".to_string()).ok();
                        info!("Oynatma başladı");
                    }
                    Command::StopPlayback => {
                        // Zaten idle; yoksay
                    }
                    Command::SetLoopCount(count) => {
                        *self.loop_count.lock().unwrap() = count;
                        info!("Loop count ayarlandı: {}", count);
                    }
                    Command::Quit => {
                        info!("Player thread kapanıyor");
                        break;
                    }
                    _ => {}
                }
            }

            if playing {
                let events = {
                    let rec = self.recording_events.lock().unwrap();
                    rec.clone()
                };

                if events.is_empty() {
                    warn!("Oynatılacak event yok!");
                    playing = false;
                    *self.state.lock().unwrap() = AppState::Idle;
                    thread::sleep(Duration::from_millis(10));
                    continue;
                }

                let loop_count = *self.loop_count.lock().unwrap();
                let mut current_loop = 0;
                let playback_start = Instant::now();
                let mut emergency_stop = false;
                // Bu oynatmada basılı duruma geçen tuşlar (stop/finish'te bırakılacak)
                let mut held: HashSet<u16> = HashSet::new();

                while !stop_flag.load(Ordering::Relaxed)
                    && (loop_count == 0 || current_loop < loop_count)
                {
                    let mut last_timestamp_us: u64 = 0;

                    for event in &events {
                        // Her event öncesi: kanal yokla (geç gelen Stop/Quit/LoopCount)
                        let (quit, _) = Self::poll_commands(
                            &self.cmd_rx,
                            &self.state,
                            &self.event_tx,
                            &self.loop_count,
                            &stop_flag,
                        );
                        if quit {
                            info!("Player thread kapanıyor (oynatma içi Quit)");
                            return;
                        }
                        if stop_flag.load(Ordering::Relaxed) {
                            emergency_stop = true;
                            break;
                        }

                        let delay_us = event.timestamp_us.saturating_sub(last_timestamp_us);
                        if delay_us > 0 {
                            // Kesilebilir bekleme: stop gelirse anında false döner
                            if !Self::precise_sleep_interruptible(
                                Duration::from_micros(delay_us),
                                &stop_flag,
                            ) {
                                emergency_stop = true;
                                break;
                            }
                        }

                        // Stop, sleep sırasında gelmiş olabilir -> emit yapma
                        if stop_flag.load(Ordering::Relaxed) {
                            emergency_stop = true;
                            break;
                        }

                        let evdev_event = event.to_evdev();
                        if let Err(e) = device.emit(&[evdev_event]) {
                            error!("Event yazılamadı: {:?}", e);
                            stop_flag.store(true, Ordering::Relaxed);
                            emergency_stop = true;
                            break;
                        }
                        Self::track_held(&mut held, event);

                        last_timestamp_us = event.timestamp_us;
                    }

                    if stop_flag.load(Ordering::Relaxed) {
                        emergency_stop = true;
                        break;
                    }

                    current_loop += 1;

                    if loop_count == 0 {
                        self.event_tx
                            .send(format!("Loop {} (sonsuz)", current_loop))
                            .ok();
                    } else {
                        self.event_tx
                            .send(format!("Loop {}/{}", current_loop, loop_count))
                            .ok();
                    }

                    // Döngü arası 50ms de kesilebilir olmalı
                    if !stop_flag.load(Ordering::Relaxed)
                        && (loop_count == 0 || current_loop < loop_count)
                        && !Self::precise_sleep_interruptible(
                            Duration::from_millis(50),
                            &stop_flag,
                        )
                    {
                        emergency_stop = true;
                        break;
                    }
                }

                let elapsed = playback_start.elapsed();
                playing = false;
                // Kritik: takılı tuş bırakılmazsa klavye/fare "bozuk" kalır
                // (örn. Ctrl basılı sanılır). Hem stop hem normal bitişte çalışır.
                let released = Self::release_held(&mut device, &mut held);
                *self.state.lock().unwrap() = AppState::Idle;
                if emergency_stop || stop_flag.load(Ordering::Relaxed) {
                    self.event_tx
                        .send(if released > 0 {
                            format!("Playback stopped ({} tuş bırakıldı)", released)
                        } else {
                            "Playback stopped".to_string()
                        })
                        .ok();
                    info!("Oynatma acil durduruldu ({:?})", elapsed);
                } else {
                    self.event_tx.send("Playback finished".to_string()).ok();
                    info!("Oynatma tamamlandı ({:?})", elapsed);
                }
                stop_flag.store(false, Ordering::Relaxed);
            }

            thread::sleep(Duration::from_millis(5));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sleep_timing_accurate_not_doubled() {
        // Eski bug ~1.9x yavaşlatıyordu; 50ms istek 45-70ms aralığında bitmeli.
        let stop = AtomicBool::new(false);
        let start = Instant::now();
        let done = Player::precise_sleep_interruptible(Duration::from_millis(50), &stop);
        let elapsed = start.elapsed();
        assert!(done);
        assert!(
            elapsed >= Duration::from_millis(45) && elapsed < Duration::from_millis(80),
            "50ms sleep {:?} sürdü (2x yavaşlık bug'ı dönmüş olabilir)",
            elapsed
        );
    }

    #[test]
    fn held_tracking_key_down_up() {
        let mut held = HashSet::new();
        let down = MacroEvent { timestamp_us: 0, event_type: 1, code: 29, value: 1 };
        let repeat = MacroEvent { timestamp_us: 10, event_type: 1, code: 29, value: 2 };
        let rel = MacroEvent { timestamp_us: 20, event_type: 2, code: 0, value: 5 };
        let up = MacroEvent { timestamp_us: 30, event_type: 1, code: 29, value: 0 };
        Player::track_held(&mut held, &down);
        Player::track_held(&mut held, &repeat);
        assert!(held.contains(&29));
        Player::track_held(&mut held, &rel); // REL dokunulmaz
        assert!(held.contains(&29));
        Player::track_held(&mut held, &up);
        assert!(held.is_empty());
    }

    #[test]
    fn sleep_interrupted_immediately_on_stop() {
        // 5ms sonra bayrak set ediliyor; 5sn'lik sleep ~50ms içinde kesilmeli.
        let stop = Arc::new(AtomicBool::new(false));
        let stop_clone = stop.clone();
        std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(5));
            stop_clone.store(true, Ordering::Relaxed);
        });
        let start = Instant::now();
        let done =
            Player::precise_sleep_interruptible(Duration::from_secs(5), &stop);
        let elapsed = start.elapsed();
        assert!(!done, "stop bayrağına rağmen sleep tamamlandı gösterdi");
        assert!(
            elapsed < Duration::from_millis(50),
            "acill stop {:?} sürdü, çok yavaş!",
            elapsed
        );
    }
}