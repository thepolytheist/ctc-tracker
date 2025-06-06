use std::collections::{HashMap, HashSet};

use eframe::egui::{self, RichText};
use log::{debug, error, info};

use crate::data::{
    db::YoutubeDatabase,
    model::{CtcVideo, VideoId},
    youtube_api::YouTubeClient,
};

/// YouTube channel ID for Cracking the Cryptic
static CHANNEL_ID: &str = "UCC-UOdK8-mIjxBQm_ot1T-Q";

/// Displays a list of videos from the Cracking the Cryptic YouTube channel with completion status.
pub struct VideoGrid {
    videos: Vec<CtcVideo>,
    video_completion_statuses: HashMap<VideoId, bool>,
    pub show_completed_videos: bool,
    pub show_without_links: bool,
    pub filter_text: String,
    yt_sender: std::sync::mpsc::Sender<Vec<CtcVideo>>,
    yt_receiver: std::sync::mpsc::Receiver<Vec<CtcVideo>>,
    completion_sender: std::sync::mpsc::Sender<HashMap<VideoId, bool>>,
    completion_receiver: std::sync::mpsc::Receiver<HashMap<VideoId, bool>>,
    yt_db: YoutubeDatabase,
    loading_completion: bool,
    completion_loaded: bool,
    loading_videos: bool,
    api_key: Option<String>,
}
impl VideoGrid {
    /// Creates a new instance of `VideoGrid`.
    pub fn new(api_key: Option<String>, yt_db: YoutubeDatabase) -> Self {
        let videos = Vec::new();
        let video_completion_statuses = HashMap::new();
        let (yt_sender, yt_receiver) = std::sync::mpsc::channel();
        let (completion_sender, completion_receiver) = std::sync::mpsc::channel();

        Self {
            videos,
            video_completion_statuses,
            show_completed_videos: false,
            show_without_links: false,
            filter_text: String::new(),
            yt_sender,
            yt_receiver,
            completion_sender,
            completion_receiver,
            yt_db,
            loading_completion: false,
            completion_loaded: false,
            loading_videos: false,
            api_key,
        }
    }

    pub fn load_completion_data(&self, ctx: egui::Context) {
        let sender = self.completion_sender.clone();
        let db = self.yt_db.clone();
        tokio::spawn(async move {
            let completion_data =
                db.get_all_video_completion_statuses()
                    .await
                    .unwrap_or_else(|e| {
                        error!("Error fetching completion statuses: {e}");
                        Vec::new()
                    });

            let mut video_completion_statuses = HashMap::new();
            for completion in completion_data {
                // Initialize the completion status for each video
                video_completion_statuses.insert(completion.id, completion.completed);
            }
            sender
                .send(video_completion_statuses)
                .expect("Failed to send completion statuses");
            ctx.request_repaint(); // Request a repaint to update the UI
        });
    }

    fn set_completion_status(&self, video_id: &VideoId, completed: bool) {
        let db = self.yt_db.clone();
        let video_id = video_id.clone();
        tokio::spawn(async move {
            if let Err(e) = db.set_video_completion_status(&video_id, completed).await {
                error!("Error updating completion status: {e}");
            }
        });
    }

    /// Loads videos from the Cracking the Cryptic YouTube channel.
    pub fn load_channel_videos(&mut self, ctx: egui::Context) {
        let sender = self.yt_sender.clone();
        let yt_client = YouTubeClient::new(self.api_key.clone().unwrap_or_default());

        // Spawn a new thread to fetch videos
        let known_video_ids: HashSet<VideoId> =
            self.video_completion_statuses.keys().cloned().collect();
        let yt_db = self.yt_db.clone();
        tokio::spawn(async move {
            let mut next_page_token = None;
            let mut videos: Vec<CtcVideo> = vec![];
            loop {
                let mut get_next_page = true;
                let result = yt_client
                    .get_channel_page(CHANNEL_ID, next_page_token)
                    .await;
                match result {
                    Ok(mut playlist_items) => {
                        // If we have any of the video IDs in the database, then we don't need to get the next page.
                        let result_video_ids =
                            crate::data::youtube_api::get_video_ids_from_playlist(
                                &mut playlist_items,
                            );
                        if result_video_ids
                            .iter()
                            .any(|id| known_video_ids.contains(id))
                        {
                            debug!(
                                "Page contains a video already in the database, skipping next fetch."
                            );
                            get_next_page = false;
                        }
                        videos.extend(
                            yt_client
                                .load_playist_videos(&mut playlist_items)
                                .await
                                .unwrap_or_else(|e: Box<dyn std::error::Error + Send + Sync>| {
                                    error!("Error loading videos from playlist: {e}");
                                    Vec::new()
                                })
                                .into_iter()
                                .filter(|video| !known_video_ids.contains(&video.id))
                                .collect::<Vec<_>>(),
                        );
                        info!("{} new videos loaded.", videos.len());
                        next_page_token = playlist_items.next_page_token.clone();
                        if !get_next_page || next_page_token.is_none() {
                            break;
                        }
                    }
                    Err(e) => {
                        error!("Error fetching next page of videos: {e}");
                        break;
                    }
                }
            }

            videos.extend(yt_db.get_all_video_data().await.unwrap_or_else(|e| {
                error!("Error fetching video data from database: {e}");
                Vec::new()
            }));

            for video in &videos {
                // Write data to DB if not already present
                if !known_video_ids.contains(&video.id) {
                    if let Err(e) = yt_db
                        .set_video_data(
                            &video.id,
                            &video.title,
                            &video.description,
                            *video.date,
                            *video.duration,
                        )
                        .await
                    {
                        error!("Error inserting video data into database: {e}");
                    }
                    if let Err(e) = yt_db.set_video_completion_status(&video.id, false).await {
                        error!("Error inserting video completion status into database: {e}");
                    }
                }
            }

            videos.sort_by(|a, b| a.duration.cmp(&b.duration));

            if sender.send(videos).is_err() {
                error!("Failed to send videos to main thread.");
            }
            ctx.request_repaint(); // Request a repaint to update the UI
        });
    }

    /// Updates the UI with the current state of the video grid.
    pub fn update(&mut self, ui: &mut egui::Ui, ctx: egui::Context) {
        if self.api_key.is_none() {
            ui.label(RichText::new("API key not set").strong());
            return;
        }

        if let Ok(completion_statuses) = self.completion_receiver.try_recv() {
            self.video_completion_statuses = completion_statuses;
            self.loading_completion = false;
            self.completion_loaded = true;
        }

        if let Ok(videos) = self.yt_receiver.try_recv() {
            self.videos = videos;
            for video in &self.videos {
                // Initialize completion status for each new video
                self.video_completion_statuses
                    .entry(video.id.clone())
                    .or_insert(false);
            }
            self.loading_videos = false;
        }

        if self.loading_completion {
            ui.label(RichText::new("Loading local data...").strong());
            return;
        } else if !self.completion_loaded {
            self.loading_completion = true;
            self.load_completion_data(ctx);
            return;
        }

        if self.loading_videos {
            ui.label(RichText::new("Loading videos...").strong());
            return;
        } else if self.videos.is_empty() {
            self.loading_videos = true;
            self.load_channel_videos(ctx);
            return;
        }

        egui::Grid::new("video_grid")
            .striped(true)
            .num_columns(6)
            .show(ui, |ui| {
                // Header row
                ui.label(RichText::new("Title").strong());
                ui.label(RichText::new("Date").strong());
                ui.label(RichText::new("Duration").strong());
                ui.label(RichText::new("Video").strong());
                ui.label(RichText::new("Puzzle").strong());
                ui.label(RichText::new("Completed").strong());
                ui.end_row();

                for video in &self.videos {
                    if !self.show_completed_videos {
                        if let Some(true) = self.video_completion_statuses.get(&video.id) {
                            continue; // Skip videos that are marked as completed
                        }
                    }

                    if !self.show_without_links && video.extracted_links.is_empty() {
                        continue; // Skip videos without links
                    }

                    if !self.filter_text.is_empty()
                        && !video
                            .title
                            .to_lowercase()
                            .contains(&self.filter_text.to_lowercase())
                    {
                        continue; // Skip videos that don't match the filter
                    }

                    ui.label(&video.title);
                    ui.label(video.date.to_string());
                    ui.label(video.duration.to_string());
                    ui.hyperlink_to("Watch video", video.get_video_url());
                    if video.extracted_links.is_empty() {
                        ui.label("No puzzle link found");
                    } else {
                        ui.hyperlink_to("Puzzle link", &video.extracted_links[0]);
                    }
                    let mut checked = self
                        .video_completion_statuses
                        .get(&video.id)
                        .cloned()
                        .unwrap_or(false);
                    if ui.checkbox(&mut checked, "").clicked() {
                        self.video_completion_statuses
                            .insert(video.id.clone(), checked);
                        // Update the database with the new completion status
                        self.set_completion_status(&video.id, checked);
                    }
                    ui.end_row();
                }
            });
    }
}
