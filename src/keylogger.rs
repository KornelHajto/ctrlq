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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyPress {
    pub key: String,
    pub timestamp: DateTime<Utc>,
    pub duration: Option<Duration>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyStats {
    pub key_counts: HashMap<String, u64>,
    pub key_sequences: Vec<String>,
    pub session_start: DateTime<Utc>,
    pub total_keystrokes: u64,
    pub typing_sessions: Vec<TypingSession>,
    pub daily_stats: HashMap<String, DayStats>, // date -> stats
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypingSession {
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
    pub keystrokes: u64,
    pub wpm: Option<f64>, // words per minute
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DayStats {
    pub keystrokes: u64,
    pub sessions: u64,
    pub most_active_hour: Option<u8>,
    pub key_distribution: HashMap<String, u64>,
}

impl KeyStats {
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

    pub fn add_keypress(&mut self, key: &str) {
        *self.key_counts.entry(key.to_string()).or_insert(0) += 1;
        self.total_keystrokes += 1;
        
        // Keep only last 100 key sequences for performance
        self.key_sequences.push(key.to_string());
        if self.key_sequences.len() > 100 {
            self.key_sequences.remove(0);
        }

        // Update daily stats
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

    pub fn get_wpm(&self) -> Option<f64> {
        if self.total_keystrokes < 5 {
            return None;
        }
        
        let elapsed = Utc::now().signed_duration_since(self.session_start);
        let minutes = elapsed.num_seconds() as f64 / 60.0;
        
        if minutes > 0.0 {
            // Rough estimation: average 5 characters per word
            Some((self.total_keystrokes as f64 / 5.0) / minutes)
        } else {
            None
        }
    }

    pub fn get_top_keys(&self, limit: usize) -> Vec<(String, u64)> {
        let mut sorted: Vec<_> = self.key_counts.iter().collect();
        sorted.sort_by(|a, b| b.1.cmp(a.1));
        sorted.into_iter()
            .take(limit)
            .map(|(k, v)| (k.clone(), *v))
            .collect()
    }
}

pub struct KeyLogger {
    device_path: String,
    stats: KeyStats,
    data_file: PathBuf,
    is_running: bool,
    key_down_times: HashMap<u16, Instant>,
}

impl KeyLogger {
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
            // Check for shutdown signal
            if shutdown_rx.has_changed().unwrap_or(false) && *shutdown_rx.borrow() {
                break;
            }

            // Check for reset signal
            if let Ok(()) = reset_rx.try_recv() {
                println!("🔄 Resetting statistics...");
                *stats = KeyStats::new();
                // Save the reset stats immediately
                let _ = Self::save_stats(stats, &data_file);
            }

            // Fetch events with timeout
            match device.fetch_events() {
                Ok(events) => {
                    for event in events {
                        // Check if this is a key event
                        if event.event_type() == evdev::EventType::KEY {
                            let key_code = event.code();
                            let key_name = format!("KEY_{}", key_code);
                            
                            match event.value() {
                                1 => {
                                    // Key down
                                    key_down_times.insert(key_code, Instant::now());
                                    stats.add_keypress(&key_name);
                                }
                                0 => {
                                    // Key up - calculate duration
                                    if let Some(down_time) = key_down_times.remove(&key_code) {
                                        let _duration = down_time.elapsed();
                                        // You could store duration data here for analysis
                                    }
                                }
                                _ => {} // Ignore autorepeat (value=2)
                            }
                        }
                    }
                }
                Err(_) => {
                    // Timeout or error, continue
                    thread::sleep(Duration::from_millis(10));
                }
            }

            // Send stats update every 100ms
            if stats_tx.send(stats.clone()).is_err() {
                break; // Receiver dropped
            }

            // Save data every 30 seconds
            if last_save.elapsed() > Duration::from_secs(30) {
                if let Err(e) = Self::save_stats(stats, &data_file) {
                    eprintln!("Failed to save stats: {}", e);
                }
                last_save = Instant::now();
            }

            thread::sleep(Duration::from_millis(10));
        }

        // Final save
        let _ = Self::save_stats(stats, &data_file);
        println!("📁 Keylogger stopped and data saved");
        Ok(())
    }

    fn save_stats(stats: &KeyStats, data_file: &PathBuf) -> Result<()> {
        let json = serde_json::to_string_pretty(stats)?;
        std::fs::write(data_file, json)?;
        Ok(())
    }

    pub fn get_stats(&self) -> &KeyStats {
        &self.stats
    }
}

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
