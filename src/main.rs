//! # CtrlQ - Developer Keylogger
//!
//! A friendly keylogger application designed for developers to analyze their typing patterns.
//! Features real-time statistics, keyboard heatmaps, and a beautiful terminal UI.
//!
//! ## Features
//! - Real-time keystroke monitoring using Linux evdev
//! - Comprehensive typing statistics and analysis
//! - Visual keyboard heatmap with color coding
//! - Data persistence across sessions (JSON format)
//! - Beautiful terminal UI with multiple tabs
//! - Reset functionality for fresh statistics
//!
//! ## Usage
//! ```bash
//! # List available keyboard devices
//! sudo ctrlq --list-devices
//!
//! # Start with automatic device detection
//! sudo ctrlq
//!
//! # Start with specific device
//! sudo ctrlq -d /dev/input/event8
//!
//! # Run without UI (headless mode)
//! sudo ctrlq --no-ui
//! ```
//!
//! ## Architecture
//! - `main.rs` - CLI interface and application coordination
//! - `keylogger.rs` - Core keystroke monitoring and statistics
//! - `ui.rs` - Terminal user interface using ratatui

mod keylogger;
mod ui;

use anyhow::Result;
use clap::{Arg, Command};
use keylogger::{find_keyboard_devices, KeyLogger};
use std::process;

/// Main entry point for the CtrlQ keylogger application.
///
/// Handles command line argument parsing, device discovery, and coordinates
/// the keylogger and UI components.
///
/// # Returns
/// `Result<()>` - Success or application error
fn main() -> Result<()> {
    let matches = Command::new("ctrlq")
        .version("0.1.0")
        .author("Developer")
        .about("A friendly keylogger for developers - generates typing statistics and heatmaps")
        .arg(
            Arg::new("device")
                .short('d')
                .long("device")
                .value_name("DEVICE_PATH")
                .help("Path to keyboard input device (e.g., /dev/input/event3)")
        )
        .arg(
            Arg::new("list-devices")
                .short('l')
                .long("list-devices")
                .action(clap::ArgAction::SetTrue)
                .help("List available keyboard devices")
        )
        .arg(
            Arg::new("no-ui")
                .long("no-ui")
                .action(clap::ArgAction::SetTrue)
                .help("Run without terminal UI (just log to file)")
        )
        .get_matches();

    if matches.get_flag("list-devices") {
        println!("🔍 Scanning for keyboard devices...\n");
        match find_keyboard_devices() {
            Ok(devices) => {
                if devices.is_empty() {
                    println!("❌ No keyboard devices found!");
                    println!("💡 Try running with sudo or check /dev/input/ permissions");
                } else {
                    println!("📱 Found {} keyboard device(s):", devices.len());
                    for (i, device) in devices.iter().enumerate() {
                        println!("  {}. {}", i + 1, device);
                    }
                    println!("\n💡 Use: ctrlq -d <device_path>");
                }
            }
            Err(e) => {
                eprintln!("❌ Error scanning devices: {}", e);
                process::exit(1);
            }
        }
        return Ok(());
    }

    let device_path = if let Some(device) = matches.get_one::<String>("device") {
        device.clone()
    } else {
        match find_keyboard_devices() {
            Ok(devices) => {
                if devices.is_empty() {
                    eprintln!("❌ No keyboard devices found!");
                    eprintln!("💡 Try: ctrlq --list-devices");
                    process::exit(1);
                } else if devices.len() == 1 {
                    println!("🎯 Auto-detected keyboard: {}", devices[0]);
                    devices[0].clone()
                } else {
                    eprintln!("❓ Multiple keyboard devices found:");
                    for (i, device) in devices.iter().enumerate() {
                        eprintln!("  {}. {}", i + 1, device);
                    }
                    eprintln!("💡 Please specify one with: ctrlq -d <device_path>");
                    process::exit(1);
                }
            }
            Err(e) => {
                eprintln!("❌ Error scanning devices: {}", e);
                eprintln!("💡 Try running with sudo or specify device manually");
                process::exit(1);
            }
        }
    };

    if !std::path::Path::new(&device_path).exists() {
        eprintln!("❌ Device not found: {}", device_path);
        eprintln!("💡 Try: ctrlq --list-devices");
        process::exit(1);
    }

    println!("🚀 Starting CtrlQ - Developer Keylogger");
    println!("📱 Device: {}", device_path);
    println!("📊 Statistics will be saved automatically");
    println!("⚠️  This tool logs keystrokes for analysis - use responsibly!");
    println!();

    let mut keylogger = KeyLogger::new(device_path)?;
    
    let (stats_rx, shutdown_tx, reset_tx) = keylogger.start_logging()?;

    if matches.get_flag("no-ui") {
        println!("⌨️  Logging keystrokes... (Press Ctrl+C to stop)");
        
        ctrlc::set_handler(move || {
            println!("\n🛑 Received Ctrl+C, shutting down...");
            let _ = shutdown_tx.send(true);
            process::exit(0);
        })?;

        while let Ok(_stats) = stats_rx.recv() {
        }
    } else {
        ui::run_ui(stats_rx, shutdown_tx, reset_tx)?;
    }

    println!("👋 CtrlQ stopped. Your keystroke data has been saved!");
    Ok(())
}
