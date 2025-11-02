use eframe::egui::{self, RichText, FontId};
use log::error;

use crate::data::db::YoutubeDatabase;

/// Result of showing the setup dialog
#[derive(Debug, Clone)]
pub enum SetupDialogResult {
    /// Dialog is still being shown
    Showing,
    /// User saved a new API key
    Saved(String),
    /// User cancelled the dialog
    Cancelled,
}

/// Setup dialog for configuring the YouTube API key.
pub struct SetupDialog {
    api_key_input: String,
    show_dialog: bool,
    save_in_progress: bool,
    db: YoutubeDatabase,
    editing_mode: bool,
    current_api_key: Option<String>,
}

impl SetupDialog {
    /// Creates a new instance of `SetupDialog` for initial setup.
    pub fn new(db: YoutubeDatabase) -> Self {
        Self {
            api_key_input: String::new(),
            show_dialog: true,
            save_in_progress: false,
            db,
            editing_mode: false,
            current_api_key: None,
        }
    }

    /// Creates a new instance of `SetupDialog` for editing an existing API key.
    pub fn new_editing(db: YoutubeDatabase, current_api_key: String) -> Self {
        Self {
            api_key_input: String::new(),
            show_dialog: true,
            save_in_progress: false,
            db,
            editing_mode: true,
            current_api_key: Some(current_api_key),
        }
    }

    /// Returns a masked version of the API key for display.
    fn mask_api_key(api_key: &str) -> String {
        if api_key.len() <= 8 {
            "*".repeat(api_key.len())
        } else {
            let visible_chars = 4;
            let masked_length = api_key.len() - visible_chars;
            format!("{}...{}", "*".repeat(masked_length.min(20)), &api_key[api_key.len() - visible_chars..])
        }
    }

    /// Shows the setup dialog and returns the result.
    pub fn show(&mut self, ctx: &egui::Context) -> SetupDialogResult {
        if !self.show_dialog {
            return if self.editing_mode {
                SetupDialogResult::Cancelled
            } else {
                SetupDialogResult::Showing
            };
        }

        let mut setup_complete = false;
        let mut api_key_to_return = None;
        let mut was_cancelled = false;

        let window_title = if self.editing_mode {
            "Edit API Key"
        } else {
            "YouTube API Key Setup"
        };

        egui::Window::new(window_title)
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(10.0);

                    if self.editing_mode {
                        ui.label(
                            RichText::new("Update API Key")
                                .font(FontId::proportional(20.0))
                        );
                        ui.add_space(10.0);

                        if let Some(ref current_key) = self.current_api_key {
                            ui.label(format!("Current API Key: {}", Self::mask_api_key(current_key)));
                            ui.add_space(10.0);
                        }
                    } else {
                        ui.label(
                            RichText::new("Welcome to CTC Tracker!")
                                .font(FontId::proportional(20.0))
                        );
                        ui.add_space(10.0);

                        ui.label("To use this application, you need to provide a YouTube API key.");
                        ui.add_space(5.0);
                    }

                    ui.label("You can get one from the Google Cloud Console:");
                    ui.hyperlink("https://console.cloud.google.com/apis/credentials");
                    ui.add_space(15.0);

                    ui.horizontal(|ui| {
                        ui.label(if self.editing_mode { "New API Key:" } else { "API Key:" });
                        ui.text_edit_singleline(&mut self.api_key_input);
                    });

                    ui.add_space(15.0);

                    ui.horizontal(|ui| {
                        if self.save_in_progress {
                            ui.label(RichText::new("Saving...").strong());
                        } else {
                            let button = ui.button(if self.editing_mode { "Update API Key" } else { "Save API Key" });
                            if button.clicked() && !self.api_key_input.trim().is_empty() {
                                let api_key = self.api_key_input.trim().to_string();
                                let db = self.db.clone();
                                self.save_in_progress = true;

                                // Save the API key to the database
                                let api_key_for_db = api_key.clone();
                                tokio::spawn(async move {
                                    if let Err(e) = db.set_api_key(&api_key_for_db).await {
                                        error!("Error saving API key: {e}");
                                    }
                                });

                                setup_complete = true;
                                api_key_to_return = Some(api_key);
                            }

                            if self.editing_mode {
                                if ui.button("Cancel").clicked() {
                                    setup_complete = true;
                                    was_cancelled = true;
                                }
                            }
                        }
                    });

                    ui.add_space(10.0);
                });
            });

        if setup_complete {
            self.show_dialog = false;
        }

        if let Some(api_key) = api_key_to_return {
            SetupDialogResult::Saved(api_key)
        } else if was_cancelled {
            SetupDialogResult::Cancelled
        } else {
            SetupDialogResult::Showing
        }
    }
}
