use std::collections::HashMap;

use eframe::egui::{self, RichText};

use crate::data::{
    db::YoutubeDatabase,
    model::{CtcVideo, VideoId},
    youtube_api::YouTubeClient,
};

/// YouTube channel ID for Cracking the Cryptic
static CHANNEL_ID: &str = "UCC-UOdK8-mIjxBQm_ot1T-Q";

/// Displays a list of videos from the Cracking the Cryptic YouTube channel with completion status.
pub struct VideoGrid {
    pub videos: Vec<CtcVideo>,
    video_completion_statuses: HashMap<VideoId, bool>,
    pub show_completed_videos: bool,
    pub show_without_links: bool,
    rt: tokio::runtime::Runtime,
    yt_sender: std::sync::mpsc::Sender<Vec<CtcVideo>>,
    yt_receiver: std::sync::mpsc::Receiver<Vec<CtcVideo>>,
    yt_db: YoutubeDatabase,
    pub loading_videos: bool,
    api_key: Option<String>,
}
impl VideoGrid {
    /// Creates a new instance of `VideoGrid`.
    pub fn new(api_key: String) -> Self {
        let videos = Vec::new();
        let mut video_completion_statuses = HashMap::new();
        let (yt_sender, yt_receiver) = std::sync::mpsc::channel();

        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        let _guard = rt.enter();

        let yt_db: YoutubeDatabase = rt.block_on(async { YoutubeDatabase::new().await });

        let completion_data = rt.block_on(async {
            yt_db
                .get_all_video_completion_statuses()
                .await
                .unwrap_or_else(|e| {
                    eprintln!("Error fetching completion statuses: {}", e);
                    Vec::new()
                })
        });

        for completion in completion_data {
            // Initialize the completion status for each video
            video_completion_statuses.insert(completion.id, completion.completed);
        }

        Self {
            videos,
            video_completion_statuses,
            show_completed_videos: false,
            show_without_links: false,
            rt,
            yt_sender,
            yt_receiver,
            yt_db,
            loading_videos: false,
            api_key: Some(api_key),
        }
    }

    /// Loads videos from the Cracking the Cryptic YouTube channel.
    pub fn load_channel_videos(&mut self) {
        let sender = self.yt_sender.clone();
        let yt_client = YouTubeClient::new(self.api_key.clone().unwrap_or_default());

        // Spawn a new thread to fetch videos
        self.rt.block_on(async{
            let mut result = yt_client.get_channel_page(CHANNEL_ID, None).await;
            let mut videos: Vec<CtcVideo> = vec![];

            match result {
                Ok(mut playlist_items) => {
                    let mut get_next_page = true;

                    // If we have any of the video IDs in the database, then we don't need to get the next page.
                    let result_video_ids = crate::data::youtube_api::get_video_ids_from_playlist(&mut playlist_items);
                    if result_video_ids.iter().any(|id| {
                        self.video_completion_statuses.contains_key(id)
                    }) {
                        println!("Page contains a video already in the database, skipping next fetch.");
                        get_next_page = false;
                    }
                    videos.extend(yt_client.load_playist_videos(&mut playlist_items).await.unwrap_or_else(|e| {
                        eprintln!("Error loading videos from playlist: {}", e);
                        Vec::new()
                    }).into_iter().filter(|video| !self.video_completion_statuses.contains_key(&video.id)).collect::<Vec<_>>());
                    println!("{} new videos loaded.", videos.len());
                    let mut next_token = playlist_items.next_page_token.clone();
                    while let Some(next_page_token) = next_token {
                        if !get_next_page {
                            break;
                        }
                        result = yt_client.get_channel_page(CHANNEL_ID, Some(next_page_token)).await;

                        match result {
                            Ok(mut next_playlist_items) => {
                                next_token = next_playlist_items.next_page_token.clone();
                                let next_video_ids = crate::data::youtube_api::get_video_ids_from_playlist(&mut next_playlist_items);
                                if next_video_ids.iter().any(|id| {
                                    self.video_completion_statuses.contains_key(id)
                                }) {
                                    println!("Page contains a video already in the database, skipping next fetch.");
                                    get_next_page = false;
                                }
                                videos.extend(yt_client.load_playist_videos(&mut next_playlist_items).await.unwrap_or_else(|e| {
                                    eprintln!("Error loading videos from next playlist page: {}", e);
                                    Vec::new()
                                }).into_iter().filter(|video| !self.video_completion_statuses.contains_key(&video.id)).collect::<Vec<_>>());
                                println!("{} new videos loaded.", videos.len());
                            }
                            Err(e) => {
                                eprintln!("Error fetching next page of videos: {}", e);
                                break;
                            }
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Error fetching initial channel page: {}", e);
                    return;
                }
            }

            videos.extend(self.yt_db.get_all_video_data().await.unwrap_or_else(|e| {
                eprintln!("Error fetching video data from database: {}", e);
                Vec::new()
            }));

            for video in &videos {
                // Write data to DB if not already present
                if !self.video_completion_statuses.contains_key(&video.id) {
                    if let Err(e) = self.yt_db.set_video_data(&video.id, &video.title, &video.description, *video.date, *video.duration).await {
                        eprintln!("Error inserting video data into database: {}", e);
                    }
                    if let Err(e) = self.yt_db.set_video_completion_status(&video.id, false).await {
                        eprintln!("Error inserting video completion status into database: {}", e);
                    }
                }
            }

            videos.sort_by(|a, b| a.duration.cmp(&b.duration));

            if sender.send(videos).is_err() {
                eprintln!("Failed to send videos to main thread.");
            }
        });
    }

    /// Updates the UI with the current state of the video grid.
    pub fn update(&mut self, ui: &mut egui::Ui) {
        if self.api_key.is_none() {
            ui.label(RichText::new("API key not set").strong());
            return;
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
                    }
                    ui.end_row();
                }
            });
    }
}
