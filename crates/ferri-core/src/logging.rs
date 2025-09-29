//! A simple file-based logger for debugging.

use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

fn get_log_path(base_path: &Path) -> io::Result<std::path::PathBuf> {
    let log_dir = base_path.join(".ferri").join("logs");
    fs::create_dir_all(&log_dir)?;
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    Ok(log_dir.join(format!("flow-run-{}.log", timestamp)))
}

pub struct FlowLogger {
    file: File,
}

impl FlowLogger {
    pub fn new(base_path: &Path) -> io::Result<Self> {
        let path = get_log_path(base_path)?;
        let file = OpenOptions::new().create(true).append(true).open(path)?;
        Ok(FlowLogger { file })
    }

    pub fn log(&mut self, message: &str) {
        let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
        let _ = writeln!(self.file, "[{}] {}", timestamp, message);
    }
}
