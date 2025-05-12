use sqlx::prelude::FromRow;

pub fn extract_links_from_description(description: &str) -> Vec<String> {
    description
        .split_whitespace()
        .filter(|word| {
            word.starts_with("http")
                && (word.contains("sudokupad.app")
                    || word.contains("crackingthecryptic.com")
                    || word.contains("cracking-the-cryptic.web.app"))
        })
        .map(|word| word.to_string())
        .collect::<Vec<_>>()
}

pub fn youtube_url_from_id(id: &str) -> String {
    format!("https://www.youtube.com/watch?v={}", id)
}

/// Represents a YouTube video ID.
#[derive(Debug, Clone, Hash, PartialEq, Eq, sqlx::Type)]
#[sqlx(transparent)]
pub struct VideoId(pub String);
impl VideoId {
    pub fn new(id: &str) -> Self {
        Self(id.to_string())
    }
}

/// Represents the publish date of a video as a Unix timestamp in milliseconds.
#[derive(Debug, Clone, Hash, PartialEq, Eq, sqlx::Type)]
#[sqlx(transparent)]
pub struct VideoPublishDate(pub i64);
impl VideoPublishDate {
    pub fn new(date: i64) -> Self {
        Self(date)
    }
}

/// Represents a YouTube video duration in seconds.
#[derive(Debug, Clone, Hash, PartialEq, Eq, sqlx::Type)]
#[sqlx(transparent)]
pub struct VideoDuration(pub u64);
impl VideoDuration {
    pub fn new(duration: u64) -> Self {
        Self(duration)
    }
}

/// Represents a video from the Cracking the Cryptic YouTube channel.
#[derive(Debug)]
pub struct CtcVideo {
    /// YouTube video ID.
    pub id: VideoId,

    /// Title of the video.
    pub title: String,

    /// Description of the video.
    pub description: String,

    /// Date when the video was published as a Unix timestamp in milliseconds.
    pub date: VideoPublishDate,

    /// Duration of the video in seconds.
    pub duration: VideoDuration,

    /// Links extracted from the video description.
    pub extracted_links: Vec<String>,
}
impl CtcVideo {
    /// Returns the YouTube URL for the video.
    pub fn get_video_url(&self) -> String {
        youtube_url_from_id(&self.id)
    }
}

#[derive(FromRow)]
pub struct CtcVideoRow {
    pub id: VideoId,
    pub title: String,
    pub description: String,
    pub date: i64,
    pub duration: u64,
}

/// Represents the completion status of a video.
#[derive(FromRow)]
pub struct CtcVideoCompletionRow {
    pub id: VideoId,
    pub completed: bool,
}
