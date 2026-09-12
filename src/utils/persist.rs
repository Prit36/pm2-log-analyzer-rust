//! Persisted UI state — one JSON document per key under [`data_dir`].
//!
//! Keys and payloads mirror the reference app's persisted store
//! (`pm2-analyzer-filters`, `app-analyzer-mode`).

use std::path::PathBuf;

/// Directory holding persisted UI state.
///
/// `PM2_ANALYZER_DATA_DIR` overrides it; the test harness sets this per run to
/// isolate persisted state.
pub fn data_dir() -> PathBuf {
    if let Some(dir) = std::env::var_os("PM2_ANALYZER_DATA_DIR") {
        return PathBuf::from(dir);
    }
    if let Some(app_data) = std::env::var_os("APPDATA") {
        return PathBuf::from(app_data).join("pm2-log-analyzer");
    }
    if let Some(config) = std::env::var_os("XDG_CONFIG_HOME") {
        return PathBuf::from(config).join("pm2-log-analyzer");
    }
    if let Some(home) = std::env::var_os("HOME") {
        return PathBuf::from(home).join(".config").join("pm2-log-analyzer");
    }
    std::env::temp_dir().join("pm2-log-analyzer")
}

fn key_path(key: &str) -> PathBuf {
    data_dir().join(format!("{key}.json"))
}

pub fn load_item(key: &str) -> Option<String> {
    std::fs::read_to_string(key_path(key)).ok()
}

pub fn set_item(key: &str, value: &str) {
    let path = key_path(key);
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let _ = std::fs::write(path, value);
}
