use std::{fmt::Display, str::FromStr};

use chrono::TimeZone;
use google_youtube3::api::Video;

use super::model::{
    extract_links_from_description, CtcVideo, CtcVideoRow, VideoDuration, VideoId, VideoPublishDate,
};

impl FromStr for VideoId {
    type Err = std::string::ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::new(s))
    }
}
impl std::fmt::Display for VideoId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
impl std::ops::Deref for VideoId {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl std::ops::DerefMut for VideoId {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<Video> for CtcVideo {
    fn from(video: Video) -> Self {
        let snippet = video.snippet.unwrap();
        let id = VideoId::new(&video.id.unwrap());
        let title = snippet.title.unwrap_or_default();
        let description = snippet.description.clone().unwrap_or_default();
        let date =
            VideoPublishDate::new(snippet.published_at.unwrap_or_default().timestamp_millis());
        let duration = VideoDuration::new(
            ::core::time::Duration::from(
                iso8601::Duration::from_str(
                    video
                        .content_details
                        .unwrap()
                        .duration
                        .unwrap_or_default()
                        .as_str(),
                )
                .unwrap_or_default(),
            )
            .as_secs(),
        );
        // Extract links from description text.
        let extracted_links = extract_links_from_description(description.as_str());
        Self {
            id,
            title,
            description,
            date,
            duration,
            extracted_links,
        }
    }
}
impl From<CtcVideoRow> for CtcVideo {
    fn from(row: CtcVideoRow) -> Self {
        Self {
            id: VideoId::new(&row.id),
            title: row.title,
            description: row.description.clone(),
            date: VideoPublishDate::new(row.date),
            duration: VideoDuration::new(row.duration),
            extracted_links: extract_links_from_description(row.description.as_str()),
        }
    }
}

impl Display for VideoPublishDate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let local_date = chrono::Utc.timestamp_opt(self.0 / 1000, 0).unwrap();
        write!(f, "{}", local_date.naive_local().format("%Y-%m-%d"))
    }
}
impl std::ops::Deref for VideoPublishDate {
    type Target = i64;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl std::ops::DerefMut for VideoPublishDate {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Display for VideoDuration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.0 {
            0..=3599 => write!(f, "{}m", self.0 / 60),
            _ => write!(f, "{}h", self.0 / 3600),
        }
    }
}
impl std::ops::Deref for VideoDuration {
    type Target = u64;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl std::ops::DerefMut for VideoDuration {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
