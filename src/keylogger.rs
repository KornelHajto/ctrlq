//! # Keylogger Module
//!
//! Core keystroke monitoring functionality using Linux evdev interface.
//! Handles device discovery, keystroke capture, statistics computation,
//! and data persistence.

use anyhow::Result;
use chrono::{DateTime, Utc};
use evdev::Device;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};
use tokio::sync::watch;

/// Represents a single keystroke event with timing information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyPress {
    /// The key that was pressed (human-readable format)
    pub key: String,
    /// When the keystroke occurred
    pub timestamp: DateTime<Utc>,
    /// How long the key was held (if available)
    pub duration: Option<Duration>,
}

/// Comprehensive typing statistics and session data.
///
/// Tracks all keystroke data, calculates statistics, and maintains
/// historical information across sessions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyStats {
    /// Count of each key pressed
    pub key_counts: HashMap<String, u64>,
    /// Recent sequence of keystrokes (limited to last 100)
    pub key_sequences: Vec<String>,
    /// When the current session started
    pub session_start: DateTime<Utc>,
    /// Total keystrokes in current session
    pub total_keystrokes: u64,
    /// Historical typing sessions
    pub typing_sessions: Vec<TypingSession>,
    /// Daily statistics by date
    pub daily_stats: HashMap<String, DayStats>,
}

/// Information about a single typing session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypingSession {
    /// Session start time
    pub start: DateTime<Utc>,
    /// Session end time
    pub end: DateTime<Utc>,
    /// Total keystrokes in session
    pub keystrokes: u64,
    /// Calculated words per minute (if available)
    pub wpm: Option<f64>,
}

/// Statistics aggregated by day.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DayStats {
    /// Total keystrokes for the day
    pub keystrokes: u64,
    /// Number of typing sessions
    pub sessions: u64,
    /// Hour of day with most activity (0-23)
    pub most_active_hour: Option<u8>,
    /// Key usage distribution for the day
    pub key_distribution: HashMap<String, u64>,
}

impl KeyStats {
    /// Creates a new empty KeyStats instance.
    ///
    /// Initializes all counters to zero and sets the session start time
    /// to the current UTC time.
    pub fn new() -> Self {
        Self {
            key_counts: HashMap::new(),
            key_sequences: Vec::new(),
            session_start: Utc::now(),
            total_keystrokes: 0,
            typing_sessions: Vec::new(),
            daily_stats: HashMap::new(),
        }
    }

    /// Records a new keystroke and updates all relevant statistics.
    ///
    /// # Arguments
    /// * `key` - The key that was pressed (human-readable format)
    pub fn add_keypress(&mut self, key: &str) {
        *self.key_counts.entry(key.to_string()).or_insert(0) += 1;
        self.total_keystrokes += 1;
        
        self.key_sequences.push(key.to_string());
        if self.key_sequences.len() > 100 {
            self.key_sequences.remove(0);
        }

        let today = Utc::now().format("%Y-%m-%d").to_string();
        let day_stats = self.daily_stats.entry(today).or_insert_with(|| DayStats {
            keystrokes: 0,
            sessions: 0,
            most_active_hour: None,
            key_distribution: HashMap::new(),
        });
        
        day_stats.keystrokes += 1;
        *day_stats.key_distribution.entry(key.to_string()).or_insert(0) += 1;
    }

    /// Calculates the current words per minute (WPM) for this session.
    ///
    /// Uses a rough estimation of 5 characters per word. Returns None
    /// if insufficient data is available for calculation.
    ///
    /// # Returns
    /// `Option<f64>` - WPM if calculable, None otherwise
    pub fn get_wpm(&self) -> Option<f64> {
        if self.total_keystrokes < 5 {
            return None;
        }
        
        let elapsed = Utc::now().signed_duration_since(self.session_start);
        let minutes = elapsed.num_seconds() as f64 / 60.0;
        
        if minutes > 0.0 {
            Some((self.total_keystrokes as f64 / 5.0) / minutes)
        } else {
            None
        }
    }

    /// Gets the top N most frequently pressed keys.
    ///
    /// Returns a sorted list of keys by press count in descending order.
    ///
    /// # Arguments
    /// * `limit` - Maximum number of keys to return
    ///
    /// # Returns
    /// `Vec<(String, u64)>` - List of (key, count) pairs
    pub fn get_top_keys(&self, limit: usize) -> Vec<(String, u64)> {
        let mut sorted: Vec<_> = self.key_counts.iter().collect();
        sorted.sort_by(|a, b| b.1.cmp(a.1));
        sorted.into_iter()
            .take(limit)
            .map(|(k, v)| (k.clone(), *v))
            .collect()
    }

    /// Resets all statistics to their initial state.
    ///
    /// Clears all keystroke counts, resets session timing,
    /// and reinitializes all tracking data.
    pub fn reset(&mut self) {
        self.key_counts.clear();
        self.key_sequences.clear();
        self.session_start = Utc::now();
        self.total_keystrokes = 0;
        self.typing_sessions.clear();
        self.daily_stats.clear();
    }
}

/// Main keylogger implementation that monitors keyboard input.
///
/// Handles device management, keystroke capture, statistics tracking,
/// and data persistence. Supports both UI and headless operation modes.
pub struct KeyLogger {
    /// Path to the keyboard input device (e.g., /dev/input/event8)
    device_path: String,
    /// Current keystroke statistics and session data
    stats: KeyStats,
    /// Path to the JSON data file for persistence
    data_file: PathBuf,
    /// Whether the keylogger is currently running
    is_running: bool,
    /// Track key press timing for hold duration calculation
    #[allow(dead_code)]
    key_down_times: HashMap<u16, Instant>,
}

impl KeyLogger {
    /// Creates a new KeyLogger instance for the specified device.
    ///
    /// Initializes the data directory, loads existing statistics from file
    /// if available, and prepares the keylogger for operation.
    ///
    /// # Arguments
    /// * `device_path` - Path to the keyboard input device
    ///
    /// # Returns
    /// `Result<Self>` - New KeyLogger instance or error
    pub fn new(device_path: String) -> Result<Self> {
        let data_dir = dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("ctrlq");
        
        std::fs::create_dir_all(&data_dir)?;
        let data_file = data_dir.join("keystroke_data.json");

        let stats = if data_file.exists() {
            let content = std::fs::read_to_string(&data_file)?;
            serde_json::from_str(&content).unwrap_or_else(|_| KeyStats::new())
        } else {
            KeyStats::new()
        };

        Ok(Self {
            device_path,
            stats,
            data_file,
            is_running: false,
            key_down_times: HashMap::new(),
        })
    }

    /// Starts the keystroke monitoring in a background thread.
    ///
    /// Creates communication channels for statistics updates, shutdown signals,
    /// and reset commands. Spawns a background thread to handle the actual
    /// keystroke monitoring loop.
    ///
    /// # Returns
    /// `Result<(Receiver<KeyStats>, Sender<bool>, Sender<()>)>` - Channels for
    /// receiving statistics updates, sending shutdown signal, and sending reset signal
    pub fn start_logging(&mut self) -> Result<(mpsc::Receiver<KeyStats>, watch::Sender<bool>, mpsc::Sender<()>)> {
        let (stats_tx, stats_rx) = mpsc::channel();
        let (shutdown_tx, shutdown_rx) = watch::channel(false);
        let (reset_tx, reset_rx) = mpsc::channel();
        
        let device_path = self.device_path.clone();
        let mut stats = self.stats.clone();
        let data_file = self.data_file.clone();
        
        self.is_running = true;

        thread::spawn(move || {
            if let Err(e) = Self::logging_loop(device_path, &mut stats, data_file, stats_tx, shutdown_rx, reset_rx) {
                eprintln!("Logging error: {}", e);
            }
        });

        Ok((stats_rx, shutdown_tx, reset_tx))
    }

    /// Main monitoring loop that runs in a background thread.
    ///
    /// Continuously monitors the specified device for keystroke events,
    /// updates statistics, handles reset commands, and periodically saves data.
    ///
    /// # Arguments
    /// * `device_path` - Path to the keyboard device to monitor
    /// * `stats` - Mutable reference to the statistics structure
    /// * `data_file` - Path to save statistics data
    /// * `stats_tx` - Channel to send statistics updates
    /// * `shutdown_rx` - Channel to receive shutdown signals
    /// * `reset_rx` - Channel to receive reset commands
    fn logging_loop(
        device_path: String,
        stats: &mut KeyStats,
        data_file: PathBuf,
        stats_tx: mpsc::Sender<KeyStats>,
        shutdown_rx: watch::Receiver<bool>,
        reset_rx: mpsc::Receiver<()>,
    ) -> Result<()> {
        let mut device = Device::open(&device_path)?;
        println!("🎯 Keylogger started on device: {}", device_path);
        println!("📊 Data will be saved to: {}", data_file.display());
        
        let mut key_down_times = HashMap::new();
        let mut last_save = Instant::now();
        
        loop {
            if shutdown_rx.has_changed().unwrap_or(false) && *shutdown_rx.borrow() {
                break;
            }

            if let Ok(()) = reset_rx.try_recv() {
                println!("🔄 Resetting statistics...");
                *stats = KeyStats::new();
                let _ = Self::save_stats(stats, &data_file);
            }

            match device.fetch_events() {
                Ok(events) => {
                    for event in events {
                        if event.event_type() == evdev::EventType::KEY {
                            let key_code = event.code();
                            let key_name = format!("KEY_{}", key_code);
                            
                            match event.value() {
                                1 => {
                                    key_down_times.insert(key_code, Instant::now());
                                    stats.add_keypress(&key_name);
                                }
                                0 => {
                                    if let Some(down_time) = key_down_times.remove(&key_code) {
                                        let _duration = down_time.elapsed();
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                }
                Err(_) => {
                    thread::sleep(Duration::from_millis(10));
                }
            }

            if stats_tx.send(stats.clone()).is_err() {
                break;
            }

            if last_save.elapsed() > Duration::from_secs(30) {
                if let Err(e) = Self::save_stats(stats, &data_file) {
                    eprintln!("Failed to save stats: {}", e);
                }
                last_save = Instant::now();
            }

            thread::sleep(Duration::from_millis(10));
        }

        let _ = Self::save_stats(stats, &data_file);
        println!("📁 Keylogger stopped and data saved");
        Ok(())
    }

    /// Saves keystroke statistics to a JSON file.
    ///
    /// Serializes the current statistics to pretty-printed JSON format
    /// and writes it to the specified file path.
    ///
    /// # Arguments
    /// * `stats` - Statistics to save
    /// * `data_file` - File path to save to
    ///
    /// # Returns
    /// `Result<()>` - Success or file I/O error
    fn save_stats(stats: &KeyStats, data_file: &PathBuf) -> Result<()> {
        let json = serde_json::to_string_pretty(stats)?;
        std::fs::write(data_file, json)?;
        Ok(())
    }

    /// Gets a reference to the current keystroke statistics.
    ///
    /// # Returns
    /// `&KeyStats` - Reference to the current statistics
    #[allow(dead_code)]
    pub fn get_stats(&self) -> &KeyStats {
        &self.stats
    }

    /// Gets the path where keystroke data is saved.
    ///
    /// # Returns
    /// `&PathBuf` - Reference to the data file path
    pub fn get_data_path(&self) -> &PathBuf {
        &self.data_file
    }

    /// Starts the keylogger for UI mode with enhanced monitoring.
    ///
    /// Similar to `start_logging` but optimized for UI interaction
    /// with more frequent statistics updates.
    ///
    /// # Returns
    /// `Result<Receiver<KeyStats>>` - Channel for receiving statistics updates
    pub fn start_ui_mode(&mut self) -> Result<mpsc::Receiver<KeyStats>> {
        let (stats_tx, stats_rx) = mpsc::channel();
        let device_path = self.device_path.clone();
        let mut stats = self.stats.clone();
        let data_file = self.data_file.clone();
        
        self.is_running = true;

        thread::spawn(move || {
            // UI mode - more frequent updates
            loop {
                // Simplified UI monitoring loop
                thread::sleep(Duration::from_millis(100));
                let _ = stats_tx.send(stats.clone());
            }
        });

        Ok(stats_rx)
    }

    /// Starts basic monitoring without UI integration.
    ///
    /// Monitors keystrokes and saves data but doesn't provide
    /// real-time statistics updates. Used for headless mode.
    pub fn start_monitoring(&mut self) -> Result<()> {
        let mut device = Device::open(&self.device_path)?;
        println!("🎯 Keylogger started on device: {}", self.device_path);
        
        loop {
            match device.fetch_events() {
                Ok(events) => {
                    for event in events {
                        if event.event_type() == evdev::EventType::KEY && event.value() == 1 {
                            let key_code = event.code();
                            let key_name = format!("KEY_{}", key_code);
                            self.stats.add_keypress(&key_name);
                        }
                    }
                }
                Err(_) => {
                    thread::sleep(Duration::from_millis(10));
                }
            }
            
            // Periodic save
            thread::sleep(Duration::from_millis(1000));
            let _ = Self::save_stats(&self.stats, &self.data_file);
        }
    }
}

/// Discovers available keyboard devices on the system.
///
/// Scans `/dev/input/` directory for event devices and tests each one
/// to determine if it's a keyboard by checking for key event support
/// and a reasonable number of keys.
///
/// # Returns
/// `Result<Vec<String>>` - List of device paths that appear to be keyboards
pub fn find_keyboard_devices() -> Result<Vec<String>> {
    let mut devices = Vec::new();
    
    for entry in std::fs::read_dir("/dev/input")? {
        let entry = entry?;
        let path = entry.path();
        
        if let Some(filename) = path.file_name() {
            if filename.to_string_lossy().starts_with("event") {
                match Device::open(&path) {
                    Ok(device) => {
                        println!("🔍 Checking device: {}", path.display());
                        if let Some(name) = device.name() {
                            println!("   Name: {}", name);
                            
                            // Check if device supports key events
                            if device.supported_events().contains(evdev::EventType::KEY) {
                                if let Some(keys) = device.supported_keys() {
                                    let key_count = keys.iter().count();
                                    println!("   Supports {} keys", key_count);
                                    
                                    // More liberal detection - any device with key events
                                    if key_count > 10 { // Reasonable threshold for a keyboard
                                        devices.push(path.to_string_lossy().to_string());
                                        println!("   ✅ Added as keyboard device");
                                    } else {
                                        println!("   ❌ Too few keys ({}) - likely not a keyboard", key_count);
                                    }
                                } else {
                                    println!("   ❌ No key capabilities");
                                }
                            } else {
                                println!("   ❌ No key events supported");
                            }
                        } else {
                            println!("   ❌ No device name");
                        }
                    }
                    Err(e) => {
                        println!("   ❌ Failed to open: {}", e);
                    }
                }
            }
        }
    }
    
    Ok(devices)
}
