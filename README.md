# CtrlQ - Professional Keystroke Analytics Tool

A comprehensive keystroke monitoring and analytics application designed for developers and typing enthusiasts. CtrlQ provides real-time typing statistics, keyboard usage patterns, and productivity insights through an intuitive terminal interface.

[![Rust](https://img.shields.io/badge/rust-stable-brightgreen.svg)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Documentation](https://img.shields.io/badge/docs-latest-blue.svg)](https://github.com/username/ctrlq/docs)

## 🚀 Features

### Real-Time Analytics
- **Live Statistics**: Monitor typing speed (WPM), keystroke count, and session duration
- **Key Frequency Analysis**: Track which keys are used most frequently
- **Session Management**: Automatic session tracking with historical data persistence

### Interactive Terminal Interface
- **Multi-Tab Interface**: Four specialized views for different analytics perspectives
- **Live Updates**: Real-time statistics refresh without manual intervention
- **Keyboard Navigation**: Intuitive controls with Tab/Shift+Tab navigation

### Visual Analytics
- **Keyboard Heatmap**: Color-coded visualization of key usage intensity
- **Usage Statistics**: Comprehensive breakdown of typing patterns
- **Activity Monitoring**: Visual gauges for typing activity levels

### Data Management
- **Automatic Persistence**: Statistics saved automatically to JSON format
- **Cross-Session Continuity**: Data preserved between application restarts
- **Reset Functionality**: Option to reset statistics for new analysis periods

## 📋 System Requirements

- **Operating System**: Linux (uses evdev interface)
- **Rust**: 1.70.0 or later
- **Permissions**: Read access to `/dev/input/` devices
- **Terminal**: Compatible with any modern terminal emulator

## 🛠️ Installation

### Prerequisites

Install Rust if not already available:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env
```

### Build from Source

```bash
git clone https://github.com/username/ctrlq.git
cd ctrlq
cargo build --release
```

The compiled binary will be available at `target/release/ctrlq`.

### Development Build

For development and testing:

```bash
cargo build
cargo run -- --help
```

## 🎯 Usage

### Basic Operation

```bash
# Run with automatic device detection
sudo ./ctrlq

# Specify a keyboard device manually
sudo ./ctrlq -d /dev/input/event8

# Run in headless mode (no UI)
sudo ./ctrlq --no-ui

# List available keyboard devices
sudo ./ctrlq --list-devices
```

### Command Line Options

| Option | Description |
|--------|-------------|
| `-d, --device <PATH>` | Specify keyboard device path |
| `--list-devices` | List all available keyboard devices |
| `--no-ui` | Run without terminal interface |
| `-h, --help` | Display help information |
| `-V, --version` | Show version information |

### Interface Controls

| Key | Action |
|-----|--------|
| `Tab` | Switch between tabs |
| `Shift+Tab` | Switch tabs in reverse |
| `r` | Reset current statistics |
| `q` | Quit application |

## 📊 Interface Overview

### 1. Overview Tab
- Session statistics summary
- Real-time WPM calculation
- Recent keystroke sequence
- Activity level indicator

### 2. Top Keys Tab
- Most frequently pressed keys
- Usage percentages
- Ranked key frequency list

### 3. Heatmap Tab
- Visual keyboard layout representation
- Color-coded key usage intensity
- QWERTY layout with usage counts

### 4. Sessions Tab
- Session history and information
- Daily statistics overview
- Historical typing data

## 🔧 Configuration

### Device Detection

CtrlQ automatically scans `/dev/input/event*` devices to identify keyboards. Manual device specification may be required in some configurations:

```bash
# Find your keyboard device
sudo ./ctrlq --list-devices

# Use specific device
sudo ./ctrlq -d /dev/input/event8
```

### Data Storage

Statistics are automatically saved to:
- **Linux**: `~/.local/share/ctrlq/keystroke_data.json`
- **Fallback**: `./keystroke_data.json` in current directory

## 🏗️ Architecture

### Core Components

- **KeyLogger Module**: Handles device monitoring and keystroke capture
- **Statistics Engine**: Processes and aggregates typing data
- **Terminal Interface**: Provides real-time visualization using ratatui
- **Data Persistence**: JSON-based statistics storage

### Dependencies

- `evdev`: Linux input device interface
- `ratatui`: Terminal user interface framework
- `tokio`: Asynchronous runtime
- `serde`: Serialization framework
- `chrono`: Date and time handling

## 🚨 Security & Privacy

### Important Considerations

- **Local Operation**: All data remains on the local system
- **No Network Activity**: No data transmission to external services
- **Permissions Required**: Requires elevated privileges to access input devices
- **Responsible Use**: Intended for personal productivity analysis only

### Privacy Protection

- Keystroke content is not stored - only statistical metadata
- No plaintext logging of typed content
- Data stored locally in user-controlled directories

## 🤝 Contributing

### Development Setup

```bash
git clone https://github.com/username/ctrlq.git
cd ctrlq
cargo build
cargo test
```

### Code Structure

```
src/
├── main.rs          # Application entry point and CLI handling
├── keylogger.rs     # Core keystroke monitoring functionality
└── ui.rs           # Terminal interface implementation
```

### Documentation

Generate and view documentation:

```bash
cargo doc --open
```

## 📈 Performance

- **Resource Usage**: Minimal CPU and memory footprint
- **Response Time**: Real-time keystroke processing with <1ms latency
- **Storage**: Efficient JSON-based data persistence
- **Scalability**: Handles extended monitoring sessions without degradation

## 🐛 Troubleshooting

### Common Issues

**Permission Denied**
```bash
# Ensure proper permissions for device access
sudo usermod -a -G input $USER
# Or run with sudo
sudo ./ctrlq
```

**No Devices Found**
```bash
# Verify device availability
ls -la /dev/input/
# Check for event devices
sudo ./ctrlq --list-devices
```

**Build Errors**
```bash
# Update Rust toolchain
rustup update stable
# Clear build cache
cargo clean && cargo build
```

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- Built with Rust and the excellent ratatui framework
- Utilizes the Linux evdev interface for keystroke monitoring
- Inspired by productivity analysis and typing improvement tools

---

**Note**: This tool is designed for personal productivity analysis. Always ensure compliance with local laws and organizational policies when monitoring keyboard input.

**Step 1:** Figure out which keyboard to spy on

```bash
sudo ./target/release/ctrlq --list-devices
```

This will dump a bunch of scary looking device paths. Pick the one that looks most keyboard-ish.

**Step 2:** Start the thing

```bash
# Let it guess your keyboard (works sometimes)
sudo ./target/release/ctrlq

# Or tell it exactly which one (recommended)
sudo ./target/release/ctrlq -d /dev/input/event8

# Run it without the fancy UI if you're boring
sudo ./target/release/ctrlq --no-ui
```

**Step 3:** Type stuff and watch the magic happen

## How to navigate this mess

- **Tab**: Switch between the four tabs (revolutionary, I know)
- **q**: Quit and return to your regularly scheduled productivity
- **r**: Reset your stats (for when the shame becomes too much)

## The four tabs of enlightenment

### 📊 Overview
The main dashboard where you realize how much time you spend deleting code you just wrote.

### 🔝 Top Keys
A leaderboard of your most abused keys. Spoiler alert: it's probably 'e' or space.

### 🔥 Heatmap
The crown jewel. A visual representation of your keyboard that shows which keys are having a bad time. Red means "this key wants to quit", gray means "I forgot this key existed".

### 📈 Sessions
Boring statistics that make you feel productive. Or unproductive. Depends on the day.

## Where it stores your typing crimes

Your keylogging data gets saved to:
- **Linux**: `~/.local/share/ctrlq/keystroke_data.json`
- **Everywhere else**: `./keystroke_data.json` (probably nowhere since this only works on Linux)

## What you'll see when it starts

```
🚀 Starting CtrlQ - Developer Keylogger
📱 Device: /dev/input/event8
📊 Statistics will be saved automatically
⚠️  This tool logs keystrokes for analysis - use responsibly!

🎯 Keylogger started on device: /dev/input/event8
📊 Data will be saved to: /home/you/.local/share/ctrlq/keystroke_data.json
```

Followed by a colorful terminal UI that updates way too frequently.

## Requirements (aka why this might not work for you)

- **Linux** (because Windows input handling is a nightmare)
- **Root privileges** (because apparently reading keyboard input requires being root)
- **A terminal that supports colors** (if you're using a black and white terminal in 2025, we need to talk)
- **Patience** (for when it inevitably breaks on your specific setup)

## Technical stuff (in case you care)

Built with:
- **Rust** 🦀 (because memory safety is cool)
- **evdev** (for talking to your keyboard at a scary low level)
- **ratatui** (for making terminals not look like 1995)
- **serde** (because JSON serialization should be easy)
- **tokio** (async stuff that probably didn't need to be async)

Probably over-engineered for what it does, but that's half the fun.

## Ethical stuff (please read this)

This tool is for:
- ✅ Figuring out your own typing patterns
- ✅ Optimizing your keyboard layout (if you're into that)
- ✅ Procrastinating from actual work
- ✅ Showing off to friends who think terminals are magic

This tool is NOT for:
- ❌ Being creepy
- ❌ Spying on other people
- ❌ Capturing passwords (seriously, don't)
- ❌ Any form of malicious activity

## Contributing (if you want to)

Got ideas? Cool. This started as a weekend project and somehow became... this. Feel free to:
- Make the heatmap even prettier
- Add more statistics that nobody asked for
- Fix the inevitable bugs I haven't found yet
- Add export features so you can share your typing shame

## License

MIT License - Do whatever you want with it. Just don't blame me when it breaks.

---

**Final warning**: This needs root to work. That's scary. You should probably read the code first to make sure I'm not doing anything sketchy. (Spoiler: I'm not, but trust nobody, including me.)

*Built during a weekend coding binge. Quality not guaranteed. Side effects may include: increased awareness of typing habits, mild obsession with keyboard statistics, and the sudden urge to optimize everything.*
