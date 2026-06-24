pub mod agents;
pub mod cli;
pub mod doctor;
pub mod install;
pub mod mneme;
pub mod opencode;
pub mod profile;
pub mod tui;

use std::path::PathBuf;

/// Get mneme-ai config directory
pub fn config_dir() -> PathBuf {
    let base = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
    base.join("mneme-ai")
}
