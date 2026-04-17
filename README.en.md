# AutoLock

A Windows auto-screen-lock tool built with Rust and eframe. Clean UI, silent background operation.

## Features

- **Timed Lock** — Custom countdown duration, auto-lock when time's up
- **System Tray** — Close window to minimize to tray; double-click tray icon to restore
- **Live Countdown** — Remaining time and progress bar; green when running, red when stopped
- **Auto Reset on Unlock** — Detects system unlock events and resets the timer automatically
- **Autostart** — Timer begins on launch, no manual action needed

## UI

Dark Catppuccin Mocha theme, easy on the eyes:

- Title: soft blue
- Countdown running: soft green / stopped: soft red
- Interactive controls: sapphire feedback

## Tech Stack

| Item | Tech |
|------|------|
| Language | Rust (Edition 2024) |
| GUI | eframe 0.33 / egui |
| Tray | tray-icon |
| Platform API | winapi |

## Requirements

- Windows 10/11
- Rust 1.85+ (Edition 2024)

## Installation

### Build from Source

```bash
git clone https://gitee.com/SamPheng/autolock.git
cd autolock
cargo build --release
```

The executable is at `target/release/autolock.exe`.

### Prebuilt Binary

Download from [Gitee Releases](https://gitee.com/SamPheng/autolock/releases).

## Usage

1. Launch — auto-starts a 25-minute countdown
2. Change duration in the input field (1–999 min) and click "Start"
3. Click "Stop" to pause while running
4. Closing the window minimizes to tray; timer keeps running
5. Double-click tray icon or right-click → "Show Window" to restore
6. Right-click tray → "Quit" to exit

## Project Structure

```
autolock/
├── src/
│   ├── main.rs        # Entry: window config, visual style, tray icon, font loading
│   ├── app.rs         # GUI: countdown display, time input, action buttons
│   ├── timer.rs       # Timer: timing, callbacks, background check thread
│   ├── platform.rs    # Platform: lock trigger, lock detection, session monitor, window restore
│   └── icon.ico       # App icon
├── build.rs           # Windows resource embedding
└── Cargo.toml
```

## Notes

- Windows only
- Recommended: add to startup for best experience

## License

MIT License
