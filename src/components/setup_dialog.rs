use eframe::egui::{self, RichText, FontId};
use log::error;

use crate::data::db::YoutubeDatabase;

/// Setup dialog for configuring the YouTube API key.
pub struct SetupDialog {
    api_key_input: String,
    show_dialog: bool,
    save_in_progress: bool,
    db: YoutubeDatabase,
}

impl SetupDialog {
    /// Creates a new instance of `SetupDialog`.
    pub fn new(db: YoutubeDatabase) -> Self {
        Self {
            api_key_input: String::new(),
            show_dialog: true,
            save_in_progress: false,
            db,
        }
    }

    /// Shows the setup dialog and returns whether the user has completed the setup.
    pub fn show(&mut self, ctx: &egui::Context) -> Option<String> {
        if !self.show_dialog {
            return None;
        }

        let mut setup_complete = false;
        let mut api_key_to_return = None;

        egui::Window::new("YouTube API Key Setup")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(10.0);
                    ui.label(
                        RichText::new("Welcome to CTC Tracker!")
                            .font(FontId::proportional(20.0))
                    );
                    ui.add_space(10.0);

                    ui.label("To use this application, you need to provide a YouTube API key.");
                    ui.add_space(5.0);
                    ui.label("You can get one from the Google Cloud Console:");
                    ui.hyperlink("https://console.cloud.google.com/apis/credentials");
                    ui.add_space(15.0);

                    ui.horizontal(|ui| {
                        ui.label("API Key:");
                        ui.text_edit_singleline(&mut self.api_key_input);
                    });

                    ui.add_space(15.0);

                    if self.save_in_progress {
                        ui.label(RichText::new("Saving...").strong());
                    } else {
                        let button = ui.button("Save API Key");
                        if button.clicked() && !self.api_key_input.trim().is_empty() {
                            let api_key = self.api_key_input.trim().to_string();
                            let db = self.db.clone();
                            self.save_in_progress = true;

                            // Save the API key to the database
                            tokio::spawn(async move {
                                if let Err(e) = db.set_api_key(&api_key).await {
                                    error!("Error saving API key: {e}");
                                }
                            });

                            setup_complete = true;
                            api_key_to_return = Some(api_key);
                        }
                    }

                    ui.add_space(10.0);
                });
            });

        if setup_complete {
            self.show_dialog = false;
        }

        api_key_to_return
    }
}
