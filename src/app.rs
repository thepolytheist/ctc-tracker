use eframe::egui::{self, FontId, RichText};

use crate::components::video_grid::VideoGrid;

pub struct CtcTrackerApp {
    error_message: Option<String>,
    video_grid: VideoGrid,
}

impl CtcTrackerApp {
    pub fn new() -> Self {
        let error_message = None;
        // Get API key from environment variable
        // TODO: Allow user to set API key in the UI
        if let Some(api_key) = std::env::var("CTC_API_KEY").ok() {
            let video_grid = VideoGrid::new(api_key);

            Self {
                error_message,
                video_grid,
            }
        } else {
            panic!("CTC_API_KEY environment variable not set.");
        }
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
                    });

                    if !self.video_grid.loading_videos
                        && self.video_grid.videos.is_empty()
                        && self.error_message.is_none()
                    {
                        println!("UI requesting video load...");
                        self.video_grid.loading_videos = true;
                        ui.label(RichText::new("Loading videos...").strong());
                        self.video_grid.load_channel_videos();
                    }

                    egui::scroll_area::ScrollArea::vertical()
                        .auto_shrink(false)
                        .show(ui, |ui| {
                            self.video_grid.update(ui);
                        });
                },
            );
        });
    }
}
