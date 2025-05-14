use std::error::Error;

use google_youtube3::{
    api::{PlaylistItemListResponse, Video},
    common::NoToken,
    hyper_rustls::{self, HttpsConnector},
    hyper_util::{self, client::legacy::connect::HttpConnector},
    YouTube,
};

use crate::data::model::CtcVideo;

use super::model::VideoId;

/// YouTube API client for fetching videos from the Cracking the Cryptic channel.
#[derive(Clone)]
pub struct YouTubeClient {
    api_key: String,
    pub hub: YouTube<HttpsConnector<HttpConnector>>,
}
impl YouTubeClient {
    /// Creates a new instance of `YouTubeClient`.
    pub fn new(api_key: String) -> Self {
        let hub = get_hub();
        Self { api_key, hub }
    }

    /// Fetches the channel page for the given channel ID.
    pub async fn get_channel_page(
        &self,
        channel_id: &str,
        page_token: Option<String>,
    ) -> Result<PlaylistItemListResponse, Box<dyn Error + Send + Sync>> {
        let mut request = self
            .hub
            .playlist_items()
            .list(&vec!["snippet".into(), "contentDetails".into()])
            .playlist_id(&get_upload_playlist(channel_id))
            .max_results(50)
            .param("key", self.api_key.as_str());

        if let Some(token) = page_token {
            request = request.page_token(&token);
        }

        match request.doit().await {
            Ok((_, response)) => Ok(response),
            Err(e) => {
                eprintln!("Error fetching channel page: {}", e);
                Err(Box::new(e))
            }
        }
    }

    /// Helper function to load videos from the playlist items response.
    pub async fn load_playist_videos(
        &self,
        playlist_items: &mut PlaylistItemListResponse,
    ) -> Result<Vec<CtcVideo>, Box<dyn Error + Send + Sync>> {
        let video_ids = get_video_ids_from_playlist(playlist_items);
        let mut video_data = Vec::<Video>::new();

        println!("Found {} videos in the playlist.", video_ids.len());

        let mut video_list_call = self
            .hub
            .videos()
            .list(&vec!["snippet".into(), "contentDetails".into()])
            .param("key", self.api_key.as_str());

        for video_id in &video_ids {
            video_list_call = video_list_call.add_id(video_id);
        }

        let video_result = video_list_call.doit().await?;

        video_data.extend(video_result.1.items.unwrap_or_default());

        let videos = video_data
            .into_iter()
            .filter(|video| {
                if video.snippet.is_some() && video.snippet.as_ref().unwrap().title.is_some() {
                    let title = video.snippet.as_ref().unwrap().title.as_ref().unwrap();
                    return !title.contains("Wordle")
                        && !title.contains("Plusword")
                        && !title.contains("Quordle");
                }
                false
            })
            .map(CtcVideo::from)
            .collect::<Vec<_>>();
        Ok(videos)
    }
}

/// Creates a new YouTube hub instance.
pub(crate) fn get_hub() -> YouTube<HttpsConnector<HttpConnector>> {
    let client = hyper_util::client::legacy::Client::builder(hyper_util::rt::TokioExecutor::new())
        .build(
            hyper_rustls::HttpsConnectorBuilder::new()
                .with_native_roots()
                .unwrap()
                .https_or_http()
                .enable_http1()
                .build(),
        );
    YouTube::new(client, NoToken)
}

/// Generates the upload playlist ID for a given channel ID.
fn get_upload_playlist(channel_id: &str) -> String {
    format!("UU{}", channel_id.chars().skip(2).collect::<String>())
}

/// Extracts video IDs from the playlist items response.
pub fn get_video_ids_from_playlist(playlist_items: &mut PlaylistItemListResponse) -> Vec<VideoId> {
    playlist_items
        .items
        .as_ref()
        .unwrap()
        .iter()
        .map(|item| &item.snippet)
        .filter(|snippet| snippet.is_some())
        .map(|snippet| &snippet.as_ref().unwrap().resource_id)
        .filter(|resource_id| resource_id.is_some())
        .map(|resource_id| &resource_id.as_ref().unwrap().video_id)
        .filter_map(|video_id| video_id.clone())
        .map(|video_id| VideoId::new(&video_id))
        .collect::<Vec<_>>()
}
