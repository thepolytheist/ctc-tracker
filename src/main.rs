use std::{path::PathBuf, sync::LazyLock};

use app::CtcTrackerApp;
use eframe::egui::{self, ViewportBuilder};

mod app;
mod components;
mod data;

/// Lazy static variable to hold the configuration directory path
/// where the database, logs, and other configuration files will be stored.
static CONFIG_DIR: LazyLock<PathBuf> = LazyLock::new(|| {
    let path = dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("Cracking the Cryptic Tracker");
    if !path.exists() {
        std::fs::create_dir_all(&path).expect("Failed to create config directory");
    }
    path
});

#[tokio::main]
async fn main() {
    // Initialize the logger with file logging
    let log_file_path = CONFIG_DIR.join("logs").join("ctc_tracker.log");
    let _log2 = log2::open(log_file_path.to_str().unwrap())
        .tee(true)
        .level(std::env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string()))
        .module_filter(|module| module.starts_with("ctc_tracker"))
        .start();

    // Initialize the database connection
    let db = data::db::YoutubeDatabase::new().await;

    // Start egui
    let options = eframe::NativeOptions {
        viewport: ViewportBuilder {
            inner_size: Some(egui::vec2(930.0, 720.0)),
            ..Default::default()
        },
        ..Default::default()
    };
    let _ = eframe::run_native(
        "Cracking the Cryptic Tracker",
        options,
        Box::new(|_cc| Ok(Box::new(CtcTrackerApp::new(db)))),
    );
}
