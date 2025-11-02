use eframe::egui::{self, FontId, RichText};
use log::error;

use crate::{
    components::{setup_dialog::{SetupDialog, SetupDialogResult}, video_grid::VideoGrid},
    data::db::YoutubeDatabase,
};

/// Main application struct for the Cracking the Cryptic Tracker.
pub struct CtcTrackerApp {
    video_grid: VideoGrid,
    setup_dialog: Option<SetupDialog>,
    api_key_receiver: std::sync::mpsc::Receiver<Option<String>>,
    api_key_loaded: bool,
}
impl CtcTrackerApp {
    pub fn new(db: YoutubeDatabase) -> Self {
        // Try to get API key from environment variable first, filtering out empty/whitespace values
        let env_api_key = std::env::var("CTC_API_KEY")
            .ok()
            .filter(|key| !key.trim().is_empty());

        // Create a channel to receive the API key from the database
        let (sender, receiver) = std::sync::mpsc::channel();

        // Spawn a task to load the API key from the database
        let db_clone = db.clone();
        tokio::spawn(async move {
            match db_clone.get_api_key().await {
                Ok(api_key) => {
                    sender.send(api_key).ok();
                }
                Err(e) => {
                    error!("Error loading API key from database: {e}");
                    sender.send(None).ok();
                }
            }
        });

        let video_grid: VideoGrid = VideoGrid::new(env_api_key, db.clone());
        let setup_dialog = Some(SetupDialog::new(db));

        Self {
            video_grid,
            setup_dialog,
            api_key_receiver: receiver,
            api_key_loaded: false,
        }
    }
}
impl eframe::App for CtcTrackerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Check if we've received the API key from the database
        if !self.api_key_loaded {
            if let Ok(db_api_key) = self.api_key_receiver.try_recv() {
                self.api_key_loaded = true;
                // Filter out empty/whitespace API keys from the database
                if let Some(api_key) = db_api_key.filter(|key| !key.trim().is_empty()) {
                    // API key found in database, use it
                    self.video_grid.set_api_key(Some(api_key));
                    self.setup_dialog = None; // No need for setup dialog
                } else if self.video_grid.has_api_key() {
                    // API key from environment variable
                    self.setup_dialog = None;
                }
            }
        }

        // Show setup dialog if API key is not set
        if let Some(setup_dialog) = &mut self.setup_dialog {
            match setup_dialog.show(ctx) {
                SetupDialogResult::Saved(api_key) => {
                    // User has entered an API key
                    self.video_grid.set_api_key(Some(api_key));
                    self.setup_dialog = None;
                }
                SetupDialogResult::Cancelled => {
                    // User cancelled - clear the error and close dialog
                    self.video_grid.api_error = None;
                    self.setup_dialog = None;
                }
                SetupDialogResult::Showing => {
                    // Dialog still showing, don't show main UI for initial setup
                    if !self.video_grid.has_api_key() {
                        return;
                    }
                }
            }
        }

        // Check if there's an API error and show settings dialog
        if self.video_grid.api_error.is_some() && self.setup_dialog.is_none() {
            // Open the settings dialog to let the user update their API key
            if let Some(api_key) = self.video_grid.api_key.clone() {
                self.setup_dialog = Some(SetupDialog::new_editing(
                    self.video_grid.yt_db.clone(),
                    api_key,
                ));
            }
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.with_layout(
                egui::Layout::top_down_justified(egui::Align::Center),
                |ui| {
                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new("Cracking the Cryptic Tracker")
                                .font(FontId::proportional(24.)),
                        );
                        let completed_video_button_text = if self.video_grid.show_completed_videos {
                            "Hide completed videos"
                        } else {
                            "Show completed videos"
                        };
                        if ui.button(completed_video_button_text).clicked() {
                            self.video_grid.show_completed_videos =
                                !self.video_grid.show_completed_videos;
                        }

                        let without_links_button_text = if self.video_grid.show_without_links {
                            "Hide videos without links"
                        } else {
                            "Show videos without links"
                        };
                        if ui.button(without_links_button_text).clicked() {
                            self.video_grid.show_without_links =
                                !self.video_grid.show_without_links;
                        }
                        ui.label(
                            RichText::new("Filter videos by:").font(FontId::proportional(16.)),
                        );
                        ui.text_edit_singleline(&mut self.video_grid.filter_text);

                        // Add refresh button
                        if ui.button("🔄 Refresh").clicked() {
                            self.video_grid.refresh_videos();
                        }

                        // Add settings button
                        if ui.button("⚙ Settings").clicked() {
                            if let Some(api_key) = self.video_grid.api_key.clone() {
                                self.setup_dialog = Some(SetupDialog::new_editing(
                                    self.video_grid.yt_db.clone(),
                                    api_key,
                                ));
                            }
                        }
                    });

                    // Show error message if there's an API error
                    if let Some(error) = self.video_grid.api_error.clone() {
                        let mut dismiss = false;
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("⚠").color(egui::Color32::RED).font(FontId::proportional(20.)));
                            ui.label(RichText::new(&error).color(egui::Color32::RED).strong());
                            if ui.button("Dismiss").clicked() {
                                dismiss = true;
                            }
                        });
                        if dismiss {
                            self.video_grid.api_error = None;
                        }
                        ui.add_space(10.0);
                    }

                    egui::scroll_area::ScrollArea::vertical()
                        .auto_shrink(false)
                        .show(ui, |ui| {
                            self.video_grid.update(ui, ctx.clone());
                        });
                },
            );
        });
    }
}
