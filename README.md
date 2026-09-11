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
- **Kernel-level recording**: captures all keyboard + mouse keys/buttons and relative motion via `/dev/input/event*`, 1ms `poll` timeout (ABS axes are recorded too but can't be replayed yet — see limitations)
- **Kernel-level playback**: event injection through a `/dev/uinput` virtual device (`LinuxTinyTask Virtual Device`)
- **Display-server independence**: works on X11/Wayland/Proton/Wine, no X11/Wayland library dependencies
- **Precise timing**: deadline-based, interruptible `precise_sleep_interruptible` (busy-wait under 100µs, chunked 2ms sleep + 800µs spin above), waiting `timestamp_us` deltas per event
- **Emergency stop**: `AtomicBool` stop flag checked before every event and ~every 1ms inside sleep; `StopPlayback` halts emission instantly, **auto-releases stuck keys** (no more broken keyboard/mouse from a wedged Ctrl) and returns to Idle (with in-playback channel polling)
- **Macro files**: save/load in `.tts` (bincode, small/fast) and `.json` (readable/debug) formats; versioned `MacroFile` wrapper, path-traversal protected
- **Macro panel**: quick Save/Load in the Control tab + a dedicated Macros tab (name, duration, event count, date)
- **Duration display**: `duration_us` computed on record, shown in the UI as `12.34s / 850ms / 400µs`
- **Loop mode**: 1–9999 repeats or infinite loop (`0 = infinite`), 50ms gap between loops
- **Playback speed**: 0.25x–4x multiplier applied to event delays and loop gaps (Control tab slider + Apply)
- **Record filter**: capture keyboard-only or mouse-only (Control tab checkboxes, at least one stays on)
- **Recent files**: last 8 macros in the Macros tab, one-click load (missing files shown greyed out)
- **Global hotkeys**: system-wide shortcuts for record/play/stop (dedicated hotkey thread)
- **Config persistence**: hotkey configuration stored as JSON in `~/.config/linux-tinytask/tinytask_config.json`
- **Minimalist UI**: `eframe/egui`, always-on-top, 420x480, tabbed interface
- **9 UI languages**: English, Türkçe, Deutsch, Français, Español, Português, Italiano, Nederlands, Polski — switchable in Settings, saved to config (more scripts need a custom font, see roadmap)
- **Multi-threaded architecture**: Dispatcher + Recorder + Player + Sync + Hotkey + UI threads communicating over `crossbeam-channel`

### Default Shortcuts
| Action | Default |
|---|---|
| Start/Stop recording (toggle) | `Ctrl+Alt+Shift+R` |
| Start playback | `Ctrl+Alt+Shift+P` |
| Stop playback | `Ctrl+Alt+Shift+S` |

Shortcuts can be changed in the Settings tab: click Change → press a single key (e.g. `F8`) or a key with `Ctrl/Alt/Shift`. Assignment takes effect immediately and is saved to disk.
> ⚠ Single-letter/key shortcuts also fire while typing — `F8–F12` recommended. The `Super` key cannot be captured (egui doesn't report it); existing Super-based shortcuts keep working.
> 🌍 The interface language is switchable in Settings (9 languages, default English) and persists across restarts.

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
│   ├── i18n.rs      # Built-in translations (9 languages) + completeness tests
│   ├── recorder.rs  # /dev/input enumeration + poll + recording (all events except SYN)
│   ├── player.rs    # uinput virtual device + precise_sleep_interruptible + loop playback
│   └── ui.rs        # eframe/egui tabs (Control / Macros / Settings / About, 9 languages)
├── Cargo.toml
├── Cargo.lock
├── LICENSE
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
- Recent stable Rust + Cargo (tested with 1.98)
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
# /dev/uinput is usually root-only: allow the input group (same as install.sh does)
echo 'KERNEL=="uinput", GROUP="input", MODE="0660"' | sudo tee /etc/udev/rules.d/99-tinytask-uinput.rules
sudo udevadm control --reload-rules && sudo udevadm trigger --subsystem-misc
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
5. The status line and event counter (`Recorded events`) show the current state.

Optional per session: tick `Keyboard`/`Mouse` in the Control tab to record only one device class; set `Speed` (0.25x–4x) + `Apply Speed` to replay faster/slower. Hotkey presses (e.g. the stop key) are automatically excluded from recordings.

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
- Use `💾 Save` / `📂 Load` in the Control tab or the Macros tab (`rfd` native dialog).
- Format by extension: `.json` → human-readable JSON, `.tts` (or other) → `bincode` binary (small/fast).
- File layout: `MacroFile { version: 1, name, created_at, duration_us, event_count, events }`. File IO happens in the recorder thread (UI never blocks); a loaded macro is instantly synced to the player copy.
- Tests: `cargo test` (16 tests: roundtrips, filters, hotkey/chord logic, recent list, sleep accuracy, stop responsiveness, key tracking, i18n completeness).

## ⚠️ Known Limitations

Honest list for the current code (details in `HANDOFF.md`):

1. **No ABS (absolute) axis playback**: the recorder captures `ABS` events but the `player.rs` virtual device only exposes keys + `REL_X/Y/WHEEL/HWHEEL`. Graphics-tablet/touchscreen absolute positions can't be replayed.
2. **Fragile sync thread**: detects the `Recording → Idle` transition by polling every 5ms; races possible on fast toggles or empty recordings.
3. **Hotkey-filter edge case**: the trigger key and its chord modifiers are stripped from recordings, but a modifier held since *before* recording started can be stripped too if it completes a hotkey chord (rare; its release is still suppressed, so no stuck keys).

## 🛣️ Roadmap
- [x] Macro save/load (JSON + bincode): file dialog + `SaveMacro/LoadMacro` implementation
- [x] Emergency stop + timing fix (interruptible sleep, in-playback channel polling)
- [x] Duration computation + UI display
- [x] Macro management panel (Macros tab)
- [x] Hotkey assignment (incl. single key: Settings → Change → press key; saved to disk)
- [x] Auto-release stuck keys on stop/finish (broken keyboard/mouse fix)
- [x] One-command install (`install.sh`) with menu entry
- [x] Multilingual UI (9 Latin-script languages, persisted)
- [x] Clean shutdown (`Quit` propagation to all threads, `join`, no `process::exit`)
- [x] Hotkey-press filtering (trigger key + chord modifiers stripped from recordings)
- [x] Playback speed multiplier (0.25x–4x)
- [x] Record filter (keyboard/mouse toggles) + recent-files list
- [ ] More UI languages (Russian/Chinese/Arabic need a bundled custom font — embedded Ubuntu-Light has no Cyrillic/CJK)
- [ ] ABS axis + `REL_Z` etc. virtual-device extension
- [ ] Playback latency tuning (per-event `emit` batching for ultra-dense macros)

## 🧪 Test Scenarios
1. **Stop**: record a 10s macro → play → stop at 2s. Expected: emission stops instantly, remaining events never play, state returns to Idle. (Manual: needs `/dev/input`+`uinput` access on a real machine.)
2. **Timing**: play a 5s macro → total playback should be 5s ±50ms. Note: pre-first-event waiting is part of the recording by design.
3. **Loop stop**: stop during infinite loop → no new loop starts, `Playback stopped` in log.
4. **Save/Load**: record → save to file → restart app → load → play. Automated: `cargo test` (16 tests passing, small roundtrips).
5. **Large macro**: record ~10,000 events → save/load/play. Must be verified manually (unit tests only cover small samples).
6. **Speed**: play the same macro at 1x and 2x → 2x run should take ~half the time (±10%).
7. **Hotkey filter**: assign single-key `F8` as stop, record via UI button, press `F8` mid-recording → saved macro must not contain F8.
8. **Shutdown**: close the window → process must exit on its own within ~1s (no kill needed); check terminal for `All threads stopped`.

## 🐛 Troubleshooting
| Symptom | Cause / Fix |
|---|---|
| `Cannot read /dev/input` | Not in `input` group → `usermod -a -G input $USER` + relogin (or run `install.sh`) |
| `Virtual device creation failed` | No `uinput` access → `install.sh` (udev rule) or run as root; check `ls -l /dev/uinput` |
| `No events to play!` | Recording empty → record first or load a file from the Macros tab (unsaved recordings reset on restart) |
| Hotkey not working | Another app may swallow the key; watch pressed codes with `RUST_LOG=debug` |
| "Does it work on Wayland?" | Yes — the app reads the kernel directly (`/dev/input`), bypassing the compositor entirely. If it fails on Wayland, it's a permission issue (see above), not a Wayland issue |
| Menu entry does nothing | `~/.local/bin` may not be in PATH or groups need relogin → log out/in, then check `which linux-tinytask` |
| Logs are in Turkish | Status messages in the UI follow the selected language; older builds logged in Turkish — current logs are English |
