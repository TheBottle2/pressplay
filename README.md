# Linux TinyTask

**Minimalist, kernel-level macro recorder and player for Linux**

Linux TinyTask is the Linux counterpart of the popular Windows TinyTask app. It talks directly to the Linux kernel input subsystem (`/dev/input` + `/dev/uinput`), independent of X11 and Wayland, recording and replaying macros with millisecond/microsecond precision.

## ⚡ Quick Start (simplest)

One-time setup (builds, installs a menu entry, fixes permissions — sudo is asked only here):

```bash
./install.sh
# if you were added to the 'input' group: log out and back in once
```

After that, launch **Linux TinyTask** from the app menu — no terminal, no sudo needed.

Alternative (without install, from this folder):

```bash
cargo build --release
./run.sh   # sudo with display env preserved (plain sudo breaks the GUI)
```

## 🎯 Features

### Working Features
- **Kernel-level recording**: captures all keyboard + mouse (REL) via `/dev/input/event*`, 1ms `poll` timeout
- **Kernel-level playback**: event injection through a `/dev/uinput` virtual device (`LinuxTinyTask Virtual Device`)
- **Display-server independence**: works on X11/Wayland/Proton/Wine, no X11/Wayland library dependencies
- **Precise timing**: deadline-based, interruptible `precise_sleep_interruptible` (busy-wait under 100µs, chunked 2ms sleep + 800µs spin above), waiting `timestamp_us` deltas per event
- **Emergency stop**: `AtomicBool` stop flag checked before every event and ~every 1ms inside sleep; `StopPlayback` halts emission instantly, **auto-releases stuck keys** (no more broken keyboard/mouse from a wedged Ctrl) and returns to Idle (with in-playback channel polling)
- **Macro files**: save/load in `.tts` (bincode, small/fast) and `.json` (readable/debug) formats; versioned `MacroFile` wrapper, path-traversal protected
- **Macro panel**: quick Save/Load in the Control tab + a dedicated `Macros` tab (name, duration, event count, date)
- **Duration display**: `duration_us` computed on record, shown in the UI as `12.34s / 850ms / 400µs`
- **Loop mode**: 1–9999 repeats or infinite loop (`0 = infinite`), 50ms gap between loops
- **Global hotkeys**: system-wide shortcuts for record/play/stop (dedicated hotkey thread)
- **Config persistence**: hotkey configuration stored as JSON in `~/.config/linux-tinytask/tinytask_config.json`
- **Minimalist UI**: `eframe/egui`, always-on-top, 420x480, tabbed interface (Control / Macros / Settings / About)
- **Multi-threaded architecture**: Dispatcher + Recorder + Player + Sync + Hotkey + UI threads communicating over `crossbeam-channel`

### Default Shortcuts
| Action | Default |
|---|---|
| Start/Stop recording (toggle) | `Ctrl+Alt+Shift+R` |
| Start playback | `Ctrl+Alt+Shift+P` |
| Stop playback | `Ctrl+Alt+Shift+S` |

Shortcuts can be changed in the **Settings** tab: click `Change` → press a single key (e.g. `F8`) or a key with `Ctrl/Alt/Shift`. Assignment takes effect immediately and is saved to disk.
> ⚠ Single-letter/key shortcuts also fire while typing — `F8–F12` recommended. The `Super` key cannot be captured (egui doesn't report it); existing Super-based shortcuts keep working.

> Note: `KeyCombo::new()` defaults to `ctrl+alt+shift` held + main key. Matching in `KeyCombo::matches()` checks left/right Ctrl/Alt/Shift/Super codes. 200ms debounce.

## 🏗️ Architecture

```
UI (egui) ──Command──▶ Dispatcher ──┬──▶ Recorder ──▶ /dev/input/event* (poll 1ms)
                                    │         ↕ (Arc<Mutex<MacroRecording>>)
                                    │     Sync thread (5ms poll, copies on Recording→Idle)
                                    │         ↓
                                    └──▶ Player ──▶ /dev/uinput (VirtualDevice)
Hotkey thread (/dev/input poll) ──Command──▶ Dispatcher
Recorder/Player ──String──▶ UI (status_message)
```

### File Structure
```
linux-tinytask/
├── src/
│   ├── main.rs      # Config load/save, dispatcher, sync thread, hotkey thread, thread spawn
│   ├── models.rs    # MacroEvent, MacroRecording, AppState, KeyCombo, HotkeyConfig, Command
│   ├── recorder.rs  # /dev/input enumeration + poll + recording (all events except SYN)
│   ├── player.rs    # uinput virtual device + precise_sleep + loop playback
│   └── ui.rs        # eframe/egui: Control / Macros / Settings / About tabs
├── Cargo.toml
├── Cargo.lock
├── install.sh           # One-time setup: build + menu entry + permissions
├── run.sh               # Run with sudo while preserving display env
├── build_appimage.sh
├── linux-tinytask.desktop
├── icon.png / icon.svg
├── README.md
└── HANDOFF.md       # Session handoff file (Turkish, updated after every change)
```

### Data Model
- `MacroEvent { timestamp_us: u64, event_type: u16, code: u16, value: i32 }` — conversion via `from_evdev`/`to_evdev`.
- `MacroRecording { name, created_at, duration_us, events: Vec<MacroEvent> }` — starts with 10,000 capacity.
- `Command`: `StartRecording | StopRecording | StartPlayback | StopPlayback | SaveMacro(String) | LoadMacro(String) | SetHotkey(HotkeyAction, KeyCombo) | SetLoopCount(u32) | SaveConfig | Quit`
- `AppState`: `Idle | Recording | Playing` — shared across threads via `Arc<Mutex<...>>`.

## 📋 Requirements

### System
- Linux kernel 4.x+, x86_64
- X11 or Wayland (doesn't matter)
- Read access to `/dev/input` and write access to `/dev/uinput` (handled by `install.sh`)

### Build
- Rust 1.70+ + Cargo
- gcc/make (build essentials)
- `libudev-dev` (Debian/Ubuntu) or `systemd-devel` (Fedora)

### Dependencies (`Cargo.toml`)
`evdev 0.12`, `nix 0.28 (poll, fs)`, `eframe/egui 0.27`, `serde + serde_json`, `bincode 1.3` (.tts format), `chrono 0.4`, `log + env_logger`, `crossbeam-channel 0.5`, `dirs 5.0`, `rfd 0.14` (native file dialog).

## 🚀 Installation

### Option A: automatic (recommended)
```bash
./install.sh
```
This builds the release binary, installs it to `~/.local/bin`, adds an app-menu entry + icon, adds you to the `input` group and installs a udev rule so `/dev/uinput` is accessible without sudo. Log out/in once if your groups changed, then launch from the menu.

### Option B: manual
```bash
# 1. Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env
rustup default stable

# 2. System deps
# Debian/Ubuntu:
sudo apt install build-essential libudev-dev
# Fedora:
sudo dnf install gcc make systemd-devel

# 3. Permissions (pick one)
sudo usermod -a -G input $USER   # then log out/in (recommended, permanent)
# or run as root with display env preserved (see below)

# 4. Build & run
cargo build --release
./target/release/linux-tinytask
# log level: RUST_LOG=debug ./target/release/linux-tinytask
```

### Running as root (GUI + sudo)
Bare `sudo` wipes the environment and the GUI can't connect to the display. Either use the group method above or carry the display variables:
```bash
./run.sh                                   # release binary via sudo, display preserved
sudo env "DISPLAY=$DISPLAY" "WAYLAND_DISPLAY=$WAYLAND_DISPLAY" \
  "XDG_RUNTIME_DIR=$XDG_RUNTIME_DIR" ./target/release/linux-tinytask
```
Note: when run as root, the config is written under `/root/.config/linux-tinytask/`.

### AppImage (portable)
```bash
chmod +x build_appimage.sh
./build_appimage.sh
chmod +x Linux_TinyTask-*.AppImage
sudo env "DISPLAY=$DISPLAY" "WAYLAND_DISPLAY=$WAYLAND_DISPLAY" \
  "XDG_RUNTIME_DIR=$XDG_RUNTIME_DIR" ./Linux_TinyTask-*.AppImage
```

## 🖥️ Usage

1. Launch the app (window stays on top).
2. **Control** tab → `● Record` (or `Ctrl+Alt+Shift+R`) → do your actions → `■ Stop`.
3. Loop setting: check `Infinite loop` or pick `1–9999` → `Apply Loop Setting` (default is `1` if never applied).
4. `▶ Play` (or `Ctrl+Alt+Shift+P`) → to stop, press **either** `■ Stop` button while playing (or `Ctrl+Alt+Shift+S`). Stopping cuts emission instantly via a shared atomic flag, independent of channel latency.
5. The status line and `Recorded events` counter show the current state.

### Config File
Path: `~/.config/linux-tinytask/tinytask_config.json`
Example:
```json
{
  "record": { "ctrl": true, "alt": true, "shift": true, "super_key": false, "key_code": 19 },
  "play":   { "ctrl": true, "alt": true, "shift": true, "super_key": false, "key_code": 25 },
  "stop":   { "ctrl": true, "alt": true, "shift": true, "super_key": false, "key_code": 31 }
}
```
Key codes are Linux evdev codes (19=R, 25=P, 31=S, 1=Esc, 57=Space, 66=F8, … — full list in `models.rs`).

### Macro Files
- Use `💾 Save` / `📂 Load` in the Control tab or the `Macros` tab (`rfd` native dialog).
- Format by extension: `.json` → human-readable JSON, `.tts` (or other) → `bincode` binary (small/fast).
- File layout: `MacroFile { version: 1, name, created_at, duration_us, event_count, events }`. File IO happens in the recorder thread (UI never blocks); a loaded macro is instantly synced to the player copy.
- Tests: `cargo test` (7 tests: json/binary roundtrip, path-traversal rejection, single-key matching, sleep accuracy, stop responsiveness, key tracking).

## ⚠️ Known Limitations

Honest list for the current code (details in `HANDOFF.md`):

1. **No ABS (absolute) axis playback**: the recorder captures `ABS` events but the `player.rs` virtual device only exposes keys + `REL_X/Y/WHEEL/HWHEEL`. Graphics-tablet/touchscreen absolute positions can't be replayed.
2. **Fragile sync thread**: detects the `Recording → Idle` transition by polling every 5ms; races possible on fast toggles or empty recordings.
3. **Unclean shutdown**: UI close calls `std::process::exit(0)`; no `Quit` propagation to threads. (In-playback Quit is handled inside the player.)
4. **Hotkey thread never exits**: infinite `loop`, doesn't listen for `Quit`; filters power/video/lid but still listens to all keyboards.
5. **Hotkey presses leak into recordings**: keys pressed for hotkeys aren't filtered from the recording (no problem when recording is started via UI button).
6. Recording while hotkeys are pressed can capture them (start recording via UI button to avoid this).

## 🛣️ Roadmap
- [x] Macro save/load (JSON + bincode): file dialog + `SaveMacro/LoadMacro` implementation
- [x] Emergency stop + timing fix (interruptible sleep, in-playback channel polling)
- [x] Duration computation + UI display
- [x] Macro management panel (Macros tab)
- [x] Hotkey assignment (incl. single key: Settings → Change → press key; saved to disk)
- [x] Auto-release stuck keys on stop/finish (broken keyboard/mouse fix)
- [x] One-command install (`install.sh`) with menu entry
- [ ] ABS axis + `REL_Z` etc. virtual-device extension
- [ ] Clean shutdown (`Quit` propagation, remove `process::exit`)
- [ ] Filter hotkey presses out of recordings
- [ ] Playback speed multiplier, latency tuning

## 🧪 Test Scenarios
1. **Stop**: record a 10s macro → play → stop at 2s. Expected: emission stops instantly, remaining events never play, state returns to Idle. (Manual: needs `/dev/input`+`uinput` access on a real machine.)
2. **Timing**: play a 5s macro → total playback should be 5s ±50ms. Note: pre-first-event waiting is part of the recording by design.
3. **Loop stop**: stop during infinite loop → no new loop starts, `Playback stopped` in log.
4. **Save/Load**: record → save to file → restart app → load → play. Automated: `cargo test` (7 tests passing).
5. **Large macro**: 10,000-event roundtrip — `cargo test` + manual recording.

## 🐛 Troubleshooting
| Symptom | Cause / Fix |
|---|---|
| `Cannot read /dev/input` | Not in `input` group → `usermod -a -G input $USER` + relogin (or run `install.sh`) |
| `Virtual device creation failed` | No `uinput` access → `install.sh` (udev rule) or run as root; check `ls -l /dev/uinput` |
| `No events to play!` | Recording empty → record first or load a file from the Macros tab (unsaved recordings reset on restart) |
| Hotkey not working | Another app may swallow the key; watch pressed codes with `RUST_LOG=debug` |
| "Does it work on Wayland?" | Yes — the app reads the kernel directly (`/dev/input`), bypassing the compositor entirely. If it fails on Wayland, it's a permission issue (see above), not a Wayland issue |
