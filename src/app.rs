use eframe::egui::{self, FontId, RichText};

use crate::{components::video_grid::VideoGrid, data::db::YoutubeDatabase};

/// Main application struct for the Cracking the Cryptic Tracker.
pub struct CtcTrackerApp {
    video_grid: VideoGrid,
}
impl CtcTrackerApp {
    pub fn new(db: YoutubeDatabase) -> Self {
        // Get API key from environment variable
        // TODO: Allow user to set API key in the UI
        let video_grid: VideoGrid = VideoGrid::new(std::env::var("CTC_API_KEY").ok(), db);
        Self { video_grid }
    }
}
impl eframe::App for CtcTrackerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
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
                            RichText::new("Filter videos by:")
                                .font(FontId::proportional(16.)),
                        );
                        ui.text_edit_singleline(&mut self.video_grid.filter_text);
                    });

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
