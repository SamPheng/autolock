# AutoLock

A Windows desktop auto-screen-lock tool developed using Rust and the eframe framework.

## Features

- **Auto Screen Lock**: Set an idle time, and the system will automatically lock the screen.
- **Tray Operation**: The application minimizes to the system tray, leaving your daily work unaffected.
- **Real-time Display**: The interface shows remaining time and a progress bar.
- **Flexible Settings**: Adjust the timer duration at any time.
- **Event Response**: Detects system unlock status and intelligently manages the timer.

## Technology Stack

- **Language**: Rust
- **GUI Framework**: eframe / egui
- **Platform**: Windows

## System Requirements

- Windows 10/11
- Rust 1.60+

## Installation

### Method 1: Compile from Source

```bash
# Clone the project
git clone https://gitee.com/SamPheng/autolock.git
cd autolock

# Build the release version
cargo build --release

# Run the program
cargo run --release
```

### Method 2: Use Precompiled Binary

Download the precompiled `autolock.exe` from [Gitee releases](https://gitee.com/SamPheng/autolock/releases).

## Usage

1. After launching the program, enter the desired lock time (in minutes) in the input field.
2. Click the "Start" button to begin the timer.
3. The interface will display the remaining time and a progress bar.
4. When the set time is reached, the screen will lock automatically.
5. Click "Reset" at any time to restart the timer.

## Project Structure

```
autolock/
├── src/
│   ├── main.rs        # Program entry point, Windows subsystem configuration
│   ├── app.rs         # Main GUI application logic
│   ├── timer.rs       # Core timer implementation
│   ├── platform.rs    # Windows platform-specific functions
│   ├── icon.ico       # Application icon
│   └── icon.svg       # SVG icon
├── build.rs           # Build script
└── Cargo.toml         # Project configuration
```

## Core Modules

### Timer (timer.rs)

The core timer component providing:
- Timer management
- Callback function configuration
- Remaining time querying
- Completion status detection

### AutolockApp (app.rs)

The egui GUI application:
- Status display
- Parameter configuration
- Interactive controls

### Platform (platform.rs)

Windows platform integration:
- Trigger screen lock (`trigger_lock`)
- Check lock status (`is_screen_locked`)
- Monitor session events (`monitor_session_events`)

## Notes

- This application is designed exclusively for the Windows platform.
- Administrator privileges are required to enable automatic screen locking.
- We recommend adding the program to startup to ensure optimal experience.

## License

This project is licensed under the MIT License.