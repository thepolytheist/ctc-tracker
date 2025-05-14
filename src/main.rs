use app::CtcTrackerApp;
use eframe::egui::{self, ViewportBuilder};

mod app;
mod components;
mod data;

#[tokio::main]
async fn main() {
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
