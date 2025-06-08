use anyhow::Result;
use evdev::Device;
use std::collections::HashMap;
use std::time::Instant;

pub struct KeyLogger {
    device_path: String,
    key_counts: HashMap<String, u64>,
    total_keystrokes: u64,
    start_time: Instant,
}

impl KeyLogger {
    pub fn new(device_path: String) -> Result<Self> {
        Ok(Self {
            device_path,
            key_counts: HashMap::new(),
            total_keystrokes: 0,
            start_time: Instant::now(),
        })
    }

    pub fn start_logging(&mut self) -> Result<()> {
        let mut device = Device::open(&self.device_path)?;
        println!("🎯 Keylogger started on device: {}", self.device_path);
        
        loop {
            match device.fetch_events() {
                Ok(events) => {
                    for event in events {
                        if event.event_type() == evdev::EventType::KEY && event.value() == 1 {
                            let key_code = event.code();
                            let key_name = format!("KEY_{}", key_code);
                            
                            *self.key_counts.entry(key_name).or_insert(0) += 1;
                            self.total_keystrokes += 1;
                            
                            println!("Key pressed: {} (Total: {})", key_code, self.total_keystrokes);
                        }
                    }
                }
                Err(_) => {
                    std::thread::sleep(std::time::Duration::from_millis(10));
                }
            }
        }
    }
}

pub fn find_keyboard_devices() -> Result<Vec<String>> {
    let mut devices = Vec::new();
    
    for entry in std::fs::read_dir("/dev/input")? {
        let entry = entry?;
        let path = entry.path();
        
        if let Some(filename) = path.file_name() {
            if filename.to_string_lossy().starts_with("event") {
                if let Ok(device) = Device::open(&path) {
                    if let Some(name) = device.name() {
                        if name.to_lowercase().contains("keyboard") || 
                           name.to_lowercase().contains("key") {
                            devices.push(path.to_string_lossy().to_string());
                        }
                    }
                }
            }
        }
    }
    
    Ok(devices)
}
