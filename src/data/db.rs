use log::{debug, info};
use sqlx::{migrate::MigrateDatabase, Executor};

use crate::CONFIG_DIR;

use super::model::{CtcVideo, CtcVideoCompletionRow, CtcVideoRow};

/// YouTube database for storing video data and completion status.
#[derive(Clone)]
pub struct YoutubeDatabase {
    pub db: sqlx::SqlitePool,
}
impl YoutubeDatabase {
    /// Creates a new instance of `YoutubeDatabase`.
    pub async fn new() -> Self {
        let db_path = CONFIG_DIR.join("db").join("ctc_tracker.db");
        let path = db_path
            .to_str()
            .expect("Config directory path should be valid.");

        // Initialize the SQLite database with sqlx
        debug!("Database path: {}", path);

        if !sqlx::Sqlite::database_exists(path).await.unwrap_or(false) {
            info!("Creating database {path}");
            match sqlx::Sqlite::create_database(path).await {
                Ok(_) => info!("Create db success"),
                Err(error) => panic!("error: {}", error),
            }
        } else {
            info!("Database already exists");
        };

        let pool = sqlx::SqlitePool::connect(path).await.unwrap_or_else(|e| {
            panic!("Failed to connect to the local database: {}", e);
        });

        // Create the video_completion table if it doesn't exist
        pool.execute("CREATE TABLE IF NOT EXISTS video_completion (id VARCHAR(10) PRIMARY KEY NOT NULL, completed BOOL NOT NULL);")
            .await
            .unwrap();

        // Create the video_data table if it doesn't exist
        pool.execute("CREATE TABLE IF NOT EXISTS video_data (id VARCHAR(10) PRIMARY KEY NOT NULL, title TEXT NOT NULL, description TEXT NOT NULL, date INTEGER NOT NULL, duration INTEGER NOT NULL);")
            .await
            .unwrap();

        // Create the settings table if it doesn't exist
        pool.execute("CREATE TABLE IF NOT EXISTS settings (key TEXT PRIMARY KEY NOT NULL, value TEXT NOT NULL);")
            .await
            .unwrap();

        Self { db: pool }
    }

    pub async fn get_all_video_completion_statuses(
        &self,
    ) -> Result<Vec<CtcVideoCompletionRow>, sqlx::Error> {
        let statuses = sqlx::query_as::<_, CtcVideoCompletionRow>(
            "SELECT id, completed FROM video_completion",
        )
        .fetch_all(&self.db)
        .await?;

        Ok(statuses)
    }

    pub async fn set_video_completion_status(
        &self,
        video_id: &str,
        completed: bool,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT INTO video_completion (id, completed) VALUES (?, ?) ON CONFLICT(id) DO UPDATE SET completed = excluded.completed"
        )
        .bind(video_id)
        .bind(completed)
        .execute(&self.db)
        .await?;

        Ok(())
    }

    /// Fetches all video data from the database.
    pub async fn get_all_video_data(&self) -> Result<Vec<CtcVideo>, sqlx::Error> {
        let rows = sqlx::query_as::<_, CtcVideoRow>(
            "SELECT id, title, description, date, duration FROM video_data",
        )
        .fetch_all(&self.db)
        .await?;

        let videos = rows.into_iter().map(CtcVideo::from).collect();

        Ok(videos)
    }

    /// Sets video data in the database.
    pub async fn set_video_data(
        &self,
        video_id: &str,
        title: &str,
        description: &str,
        date: i64,
        duration: u64,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT INTO video_data (id, title, description, date, duration) VALUES (?, ?, ?, ?, ?) ON CONFLICT(id) DO UPDATE SET title = excluded.title, description = excluded.description, date = excluded.date, duration = excluded.duration"
        )
        .bind(video_id)
        .bind(title)
        .bind(description)
        .bind(date)
        .bind(duration as i64)
        .execute(&self.db)
        .await?;

        Ok(())
    }

    /// Gets the API key from the database.
    pub async fn get_api_key(&self) -> Result<Option<String>, sqlx::Error> {
        let result = sqlx::query_as::<_, (String,)>(
            "SELECT value FROM settings WHERE key = 'api_key'"
        )
        .fetch_optional(&self.db)
        .await?;

        Ok(result.map(|(value,)| value))
    }

    /// Sets the API key in the database.
    pub async fn set_api_key(&self, api_key: &str) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT INTO settings (key, value) VALUES ('api_key', ?) ON CONFLICT(key) DO UPDATE SET value = excluded.value"
        )
        .bind(api_key)
        .execute(&self.db)
        .await?;

        Ok(())
    }
}
