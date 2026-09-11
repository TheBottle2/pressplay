use evdev::{EventType, InputEvent};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct MacroEvent {
    pub timestamp_us: u64,
    pub event_type: u16,
    pub code: u16,
    pub value: i32,
}

impl MacroEvent {
    pub fn from_evdev(event: &InputEvent, start_time: SystemTime) -> Self {
        let timestamp_us = event
            .timestamp()
            .duration_since(start_time)
            .unwrap_or_default()
            .as_micros() as u64;

        Self {
            timestamp_us,
            event_type: event.event_type().0,
            code: event.code(),
            value: event.value(),
        }
    }

    pub fn to_evdev(&self) -> InputEvent {
        InputEvent::new(EventType(self.event_type), self.code, self.value)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MacroRecording {
    pub name: String,
    pub created_at: u64,
    pub duration_us: u64,
    pub events: Vec<MacroEvent>,
}

impl MacroRecording {
    pub fn new(name: String) -> Self {
        let created_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Self {
            name,
            created_at,
            duration_us: 0,
            events: Vec::with_capacity(10_000),
        }
    }

    /// Süreyi insan-okunur formatta döndürür ("12.34s" / "850ms")
    pub fn duration_string(&self) -> String {
        Self::format_duration(self.duration_us)
    }

    pub fn format_duration(duration_us: u64) -> String {
        if duration_us >= 1_000_000 {
            format!("{:.2}s", duration_us as f64 / 1_000_000.0)
        } else if duration_us >= 1_000 {
            format!("{:.1}ms", duration_us as f64 / 1_000.0)
        } else {
            format!("{}µs", duration_us)
        }
    }

    /// Dosya uzantısına göre format seçer: .json -> JSON, diğer (.tts) -> bincode
    pub fn save_to_file(&self, path: &str) -> Result<(), String> {
        // Path traversal koruması: mutlak yolu normalize et, parent erişimini reddet
        if path.contains("..") {
            return Err("Invalid file path (must not contain ..)".to_string());
        }
        let file = MacroFile {
            version: MacroFile::CURRENT_VERSION,
            name: self.name.clone(),
            created_at: self.created_at,
            duration_us: self.duration_us,
            event_count: self.events.len() as u32,
            events: self.events.clone(),
        };
        if path.ends_with(".json") {
            let data = serde_json::to_string_pretty(&file)
                .map_err(|e| format!("JSON write error: {}", e))?;
            std::fs::write(path, data).map_err(|e| format!("Cannot write file: {}", e))?;
        } else {
            let data =
                bincode::serialize(&file).map_err(|e| format!("Binary write error: {}", e))?;
            std::fs::write(path, data).map_err(|e| format!("Cannot write file: {}", e))?;
        }
        Ok(())
    }

    pub fn load_from_file(path: &str) -> Result<Self, String> {
        if path.contains("..") {
            return Err("Invalid file path (must not contain ..)".to_string());
        }
        let data = std::fs::read(path).map_err(|e| format!("Cannot read file: {}", e))?;
        let file: MacroFile = if path.ends_with(".json") {
            serde_json::from_slice(&data).map_err(|e| format!("JSON parse error: {}", e))?
        } else {
            // Geriye uyumluluk: önce yeni MacroFile formatını dene,
            // başarısız olursa saf Vec<MacroEvent> (eski bincode) dene
            match bincode::deserialize::<MacroFile>(&data) {
                Ok(f) => f,
                Err(_) => {
                    let events: Vec<MacroEvent> = bincode::deserialize(&data)
                        .map_err(|e| format!("Binary parse error: {}", e))?;
                    let duration_us = events.last().map(|e| e.timestamp_us).unwrap_or(0);
                    MacroFile {
                        version: 1,
                        name: "imported".to_string(),
                        created_at: SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .map(|d| d.as_secs())
                            .unwrap_or(0),
                        duration_us,
                        event_count: events.len() as u32,
                        events,
                    }
                }
            }
        };
        if file.version > MacroFile::CURRENT_VERSION {
            return Err(format!(
                "Unsupported file version: {} (max {})",
                file.version,
                MacroFile::CURRENT_VERSION
            ));
        }
        Ok(Self {
            name: file.name,
            created_at: file.created_at,
            duration_us: file.duration_us,
            events: file.events,
        })
    }
}

/// Disk formatı: versiyonlu sarmalayıcı (JSON ve bincode ile aynı struct kullanılır)
#[derive(Debug, Serialize, Deserialize)]
pub struct MacroFile {
    pub version: u32,
    pub name: String,
    pub created_at: u64,
    pub duration_us: u64,
    pub event_count: u32,
    pub events: Vec<MacroEvent>,
}

impl MacroFile {
    pub const CURRENT_VERSION: u32 = 1;
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_recording() -> MacroRecording {
        let mut rec = MacroRecording::new("test".to_string());
        rec.events = vec![
            MacroEvent { timestamp_us: 100_000, event_type: 1, code: 30, value: 1 },
            MacroEvent { timestamp_us: 150_000, event_type: 1, code: 30, value: 0 },
        ];
        rec.duration_us = 150_000;
        rec
    }

    #[test]
    fn json_roundtrip() {
        let rec = sample_recording();
        let path = "/tmp/tt_test_macro.json";
        rec.save_to_file(path).unwrap();
        let loaded = MacroRecording::load_from_file(path).unwrap();
        assert_eq!(loaded.events.len(), 2);
        assert_eq!(loaded.duration_us, 150_000);
        assert_eq!(loaded.name, "test");
        std::fs::remove_file(path).ok();
    }

    #[test]
    fn binary_roundtrip() {
        let rec = sample_recording();
        let path = "/tmp/tt_test_macro.tts";
        rec.save_to_file(path).unwrap();
        let loaded = MacroRecording::load_from_file(path).unwrap();
        assert_eq!(loaded.events.len(), 2);
        assert_eq!(loaded.events[1].code, 30);
        std::fs::remove_file(path).ok();
    }

    #[test]
    fn rejects_path_traversal() {
        let rec = sample_recording();
        assert!(rec.save_to_file("/tmp/../etc/evil.tts").is_err());
        assert!(MacroRecording::load_from_file("../evil.tts").is_err());
    }

    #[test]
    fn single_key_combo_matches() {
        use std::collections::HashSet;
        // Tek tuş: modifier yok, sadece F8 (evdev 66)
        let combo = KeyCombo { ctrl: false, alt: false, shift: false, super_key: false, key_code: 66 };
        assert_eq!(combo.to_string(), "F8");
        let mut pressed = HashSet::new();
        pressed.insert(66u16);
        assert!(combo.matches(&pressed));
        // Ctrl ile birlikte basılırsa EŞLEŞMEMELİ (katı eşleşme)
        pressed.insert(29u16);
        assert!(!combo.matches(&pressed));
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppState {
    Idle,
    Recording,
    Playing,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HotkeyAction {
    Record,
    Play,
    Stop,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyCombo {
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,
    pub super_key: bool,
    pub key_code: u16,
}

impl KeyCombo {
    pub fn new(key_code: u16) -> Self {
        Self {
            ctrl: true,
            alt: true,
            shift: true,
            super_key: false,
            key_code,
        }
    }

    pub fn to_string(&self) -> String {
        let mut result = String::new();
        if self.ctrl {
            result.push_str("Ctrl+");
        }
        if self.alt {
            result.push_str("Alt+");
        }
        if self.shift {
            result.push_str("Shift+");
        }
        if self.super_key {
            result.push_str("Super+");
        }

        let key_name = match self.key_code {
            19 => "R",
            25 => "P",
            31 => "S",
            28 => "Enter",
            57 => "Space",
            14 => "Backspace",
            15 => "Tab",
            1 => "Esc",
            30 => "A",
            48 => "B",
            46 => "C",
            32 => "D",
            18 => "E",
            33 => "F",
            34 => "G",
            35 => "H",
            23 => "I",
            36 => "J",
            37 => "K",
            38 => "L",
            50 => "M",
            49 => "N",
            24 => "O",
            16 => "Q",
            20 => "T",
            22 => "U",
            47 => "V",
            17 => "W",
            45 => "X",
            21 => "Y",
            44 => "Z",
            59 => "F1",
            60 => "F2",
            61 => "F3",
            62 => "F4",
            63 => "F5",
            64 => "F6",
            65 => "F7",
            66 => "F8",
            67 => "F9",
            68 => "F10",
            87 => "F11",
            88 => "F12",
            103 => "Up",
            108 => "Down",
            105 => "Left",
            106 => "Right",
            102 => "Home",
            107 => "End",
            104 => "PageUp",
            109 => "PageDown",
            110 => "Insert",
            111 => "Delete",
            _ => {
                result.push_str(&format!("Key{}", self.key_code));
                return result;
            }
        };
        result.push_str(key_name);
        result
    }

    pub fn matches(&self, pressed: &std::collections::HashSet<u16>) -> bool {
        let ctrl_pressed = pressed.contains(&29) || pressed.contains(&97);
        let alt_pressed = pressed.contains(&56) || pressed.contains(&100);
        let shift_pressed = pressed.contains(&42) || pressed.contains(&54);
        let super_pressed = pressed.contains(&125) || pressed.contains(&126);

        let ctrl_match = self.ctrl == ctrl_pressed;
        let alt_match = self.alt == alt_pressed;
        let shift_match = self.shift == shift_pressed;
        let super_match = self.super_key == super_pressed;
        let key_match = pressed.contains(&self.key_code);

        ctrl_match && alt_match && shift_match && super_match && key_match
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HotkeyConfig {
    pub record: KeyCombo,
    pub play: KeyCombo,
    pub stop: KeyCombo,
    /// UI language code ("en", "tr", ...). Defaulted so old config files still parse.
    #[serde(default = "default_lang_code")]
    pub lang: String,
}

fn default_lang_code() -> String {
    "en".to_string()
}

impl Default for HotkeyConfig {
    fn default() -> Self {
        Self {
            record: KeyCombo::new(19), // R
            play: KeyCombo::new(25),   // P
            stop: KeyCombo::new(31),   // S
            lang: default_lang_code(),
        }
    }
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum Command {
    StartRecording,
    StopRecording,
    StartPlayback,
    StopPlayback,
    SaveMacro(String),
    LoadMacro(String),
    SetHotkey(HotkeyAction, KeyCombo),
    SetLoopCount(u32),
    SaveConfig,
    Quit,
}