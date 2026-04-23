use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use std::path::Path;

use crate::search::huffman::{self, HuffmanError};

#[derive(Debug, thiserror::Error)]
pub enum DbError {
    #[error("database error: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("migration error: {0}")]
    Migrate(#[from] sqlx::migrate::MigrateError),
    #[error("io error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("huffman error: {0}")]
    Huffman(#[from] HuffmanError),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Clip {
    pub id: i64,
    pub content: String,
    pub content_type: String,
    pub created_at: String,
    pub source_app: Option<String>,
    pub pinned: bool,
    pub deleted_at: Option<String>,
    pub was_compressed: bool,
    pub original_size: i64,
    pub compressed_size: i64,
}

#[derive(Debug, Clone, sqlx::FromRow)]
struct ClipRow {
    id: i64,
    content: String,
    content_type: String,
    created_at: String,
    source_app: Option<String>,
    pinned: bool,
    deleted_at: Option<String>,
    was_compressed: bool,
    original_size: i64,
    compressed_size: i64,
}

impl ClipRow {
    fn decode(self) -> Result<Clip, DbError> {
        let decoded = huffman::maybe_decode(self.content.as_bytes(), self.was_compressed)?;
        Ok(Clip {
            id: self.id,
            content: decoded,
            content_type: self.content_type,
            created_at: self.created_at,
            source_app: self.source_app,
            pinned: self.pinned,
            deleted_at: self.deleted_at,
            was_compressed: self.was_compressed,
            original_size: self.original_size,
            compressed_size: self.compressed_size,
        })
    }
}

pub struct Db {
    pool: SqlitePool,
}

impl Db {
    pub async fn new(app_data_dir: &Path) -> Result<Self, DbError> {
        std::fs::create_dir_all(app_data_dir)?;

        let db_path = app_data_dir.join("synaptclip.db");
        let db_url = format!("sqlite:{}?mode=rwc", db_path.display());

        let pool = SqlitePool::connect(&db_url).await?;
        sqlx::migrate!("./migrations").run(&pool).await?;

        tracing::info!("Database initialized at {}", db_path.display());
        Ok(Self { pool })
    }

    pub async fn insert_clip(
        &self,
        content: &str,
        content_type: &str,
        source_app: Option<&str>,
    ) -> Result<Clip, DbError> {
        let original_size = content.len() as i64;
        let (payload, was_compressed) = huffman::maybe_encode(content);
        let compressed_size = payload.len() as i64;
        let payload_text = String::from_utf8(payload)
            .map_err(|e| DbError::Huffman(HuffmanError::Malformed(format!("utf-8: {e}"))))?;

        let row = sqlx::query_as::<_, ClipRow>(
            "INSERT INTO clips (content, content_type, source_app, was_compressed, original_size, compressed_size) \
             VALUES (?, ?, ?, ?, ?, ?) \
             RETURNING id, content, content_type, created_at, source_app, pinned, deleted_at, \
                       was_compressed, original_size, compressed_size",
        )
        .bind(&payload_text)
        .bind(content_type)
        .bind(source_app)
        .bind(was_compressed)
        .bind(original_size)
        .bind(compressed_size)
        .fetch_one(&self.pool)
        .await?;

        Ok(Clip {
            id: row.id,
            content: content.to_string(),
            content_type: row.content_type,
            created_at: row.created_at,
            source_app: row.source_app,
            pinned: row.pinned,
            deleted_at: row.deleted_at,
            was_compressed: row.was_compressed,
            original_size: row.original_size,
            compressed_size: row.compressed_size,
        })
    }

    pub async fn get_recent_clips(&self, limit: i64) -> Result<Vec<Clip>, DbError> {
        let rows = sqlx::query_as::<_, ClipRow>(
            "SELECT id, content, content_type, created_at, source_app, pinned, deleted_at, \
                    was_compressed, original_size, compressed_size \
             FROM clips WHERE deleted_at IS NULL ORDER BY created_at DESC LIMIT ?",
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(|r| r.decode()).collect()
    }

    pub async fn get_last_clip_content(&self) -> Result<Option<String>, DbError> {
        let row: Option<(String, bool)> = sqlx::query_as(
            "SELECT content, was_compressed FROM clips WHERE deleted_at IS NULL \
             ORDER BY created_at DESC LIMIT 1",
        )
        .fetch_optional(&self.pool)
        .await?;

        match row {
            Some((raw, was_compressed)) => {
                let decoded = huffman::maybe_decode(raw.as_bytes(), was_compressed)?;
                Ok(Some(decoded))
            }
            None => Ok(None),
        }
    }

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

    pub async fn soft_delete_clip(&self, id: i64) -> Result<(), DbError> {
        sqlx::query("UPDATE clips SET deleted_at = datetime('now') WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    /// Clear the `deleted_at` timestamp for a previously soft-deleted clip.
    pub async fn restore_clip(&self, id: i64) -> Result<Option<Clip>, DbError> {
        sqlx::query("UPDATE clips SET deleted_at = NULL WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;

        self.get_clip_by_id(id).await
    }

    pub async fn hard_delete_clip(&self, id: i64) -> Result<(), DbError> {
        sqlx::query("DELETE FROM clips WHERE id = ? AND pinned = 0")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn clear_all_clips(&self) -> Result<(), DbError> {
        sqlx::query(
            "UPDATE clips SET deleted_at = datetime('now') WHERE pinned = 0 AND deleted_at IS NULL",
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_clip_by_id(&self, id: i64) -> Result<Option<Clip>, DbError> {
        let row = sqlx::query_as::<_, ClipRow>(
            "SELECT id, content, content_type, created_at, source_app, pinned, deleted_at, \
                    was_compressed, original_size, compressed_size \
             FROM clips WHERE id = ?",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        match row {
            Some(r) => Ok(Some(r.decode()?)),
            None => Ok(None),
        }
    }

    pub async fn is_pinned(&self, id: i64) -> Result<bool, DbError> {
        let row: Option<(bool,)> = sqlx::query_as("SELECT pinned FROM clips WHERE id = ?")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;
        Ok(row.map(|r| r.0).unwrap_or(false))
    }
}
