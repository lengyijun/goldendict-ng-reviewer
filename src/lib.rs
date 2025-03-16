pub mod fsrs;
pub mod utils;

use anyhow::Context;
use std::fs::create_dir;
use std::path::PathBuf;

pub fn dictionary_dir() -> PathBuf {
    // ~/.local/share/goldendict
    let path = dirs::data_local_dir().unwrap().join("goldendict");
    if !path.exists() {
        create_dir(&path)
            .with_context(|| format!("Failed to create directory {:?}", path))
            .unwrap();
    }
    path
}

pub fn db_path() -> PathBuf {
    dictionary_dir().join("history.db")
}

pub fn log_dir() -> PathBuf {
    let path = dirs::cache_dir().unwrap().join("goldendict");
    if !path.exists() {
        create_dir(&path)
            .with_context(|| format!("Failed to create directory {:?}", path))
            .unwrap();
    }
    path
}
