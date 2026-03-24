use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use std::path::Path;

/// Error types for database operations.
#[derive(Debug, thiserror::Error)]
pub enum DbError {
    /// SQLx query or connection error.
    #[error("database error: {0}")]
    Sqlx(#[from] sqlx::Error),
    /// Migration error during database initialization.
    #[error("migration error: {0}")]
    Migrate(#[from] sqlx::migrate::MigrateError),
    /// Filesystem I/O error.
    #[error("io error: {0}")]
    IoError(#[from] std::io::Error),
}

/// A single clipboard entry.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Clip {
    /// Unique identifier.
    pub id: i64,
    /// The clipboard text content.
    pub content: String,
    /// Content type — always "text" in v0.1.
    pub content_type: String,
    /// ISO 8601 timestamp of when the clip was captured.
    pub created_at: String,
    /// Application that produced the clip (None in v0.1).
    pub source_app: Option<String>,
    /// Whether the clip is pinned.
    pub pinned: bool,
    /// Soft-delete timestamp, if deleted.
    pub deleted_at: Option<String>,
}

/// Database handle wrapping a SQLite connection pool.
pub struct Db {
    pool: SqlitePool,
}

impl Db {
    /// Open (or create) the database at `app_data_dir/synaptclip.db`
    /// and run pending migrations.
    pub async fn new(app_data_dir: &Path) -> Result<Self, DbError> {
        std::fs::create_dir_all(app_data_dir)?;

        let db_path = app_data_dir.join("synaptclip.db");
        let db_url = format!("sqlite:{}?mode=rwc", db_path.display());

        let pool = SqlitePool::connect(&db_url).await?;
        sqlx::migrate!("./migrations").run(&pool).await?;

        tracing::info!("Database initialized at {}", db_path.display());
        Ok(Self { pool })
    }

    /// Insert a new clip and return the stored record.
    pub async fn insert_clip(
        &self,
        content: &str,
        content_type: &str,
        source_app: Option<&str>,
    ) -> Result<Clip, DbError> {
        let row = sqlx::query_as::<_, Clip>(
            "INSERT INTO clips (content, content_type, source_app) \
             VALUES (?, ?, ?) \
             RETURNING id, content, content_type, created_at, source_app, pinned, deleted_at",
        )
        .bind(content)
        .bind(content_type)
        .bind(source_app)
        .fetch_one(&self.pool)
        .await?;

        Ok(row)
    }

    /// Fetch the most recent non-deleted clips, ordered newest first.
    pub async fn get_recent_clips(&self, limit: i64) -> Result<Vec<Clip>, DbError> {
        let clips = sqlx::query_as::<_, Clip>(
            "SELECT id, content, content_type, created_at, source_app, pinned, deleted_at \
             FROM clips WHERE deleted_at IS NULL ORDER BY created_at DESC LIMIT ?",
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(clips)
    }

    /// Get the content of the most recent non-deleted clip for duplicate detection.
    pub async fn get_last_clip_content(&self) -> Result<Option<String>, DbError> {
        let row: Option<(String,)> = sqlx::query_as(
            "SELECT content FROM clips WHERE deleted_at IS NULL \
             ORDER BY created_at DESC LIMIT 1",
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| r.0))
    }

    /// Delete the oldest non-pinned clips beyond the given limit.
    pub async fn enforce_history_limit(&self, limit: i64) -> Result<(), DbError> {
        sqlx::query(
            "DELETE FROM clips WHERE id IN (\
                SELECT id FROM clips WHERE pinned = 0 AND deleted_at IS NULL \
                ORDER BY created_at DESC LIMIT -1 OFFSET ?\
             )",
        )
        .bind(limit)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Soft-delete a clip by setting its deleted_at timestamp.
    pub async fn soft_delete_clip(&self, id: i64) -> Result<(), DbError> {
        sqlx::query("UPDATE clips SET deleted_at = datetime('now') WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    /// Soft-delete all non-pinned clips.
    pub async fn clear_all_clips(&self) -> Result<(), DbError> {
        sqlx::query(
            "UPDATE clips SET deleted_at = datetime('now') WHERE pinned = 0 AND deleted_at IS NULL",
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Get a single clip by ID.
    pub async fn get_clip_by_id(&self, id: i64) -> Result<Option<Clip>, DbError> {
        let clip = sqlx::query_as::<_, Clip>(
            "SELECT id, content, content_type, created_at, source_app, pinned, deleted_at \
             FROM clips WHERE id = ?",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(clip)
    }
}
