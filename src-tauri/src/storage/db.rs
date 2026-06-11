use std::collections::HashMap;
use std::path::Path;

use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

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
    pub category: Option<String>,
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
    category: Option<String>,
}

const CLIP_COLUMNS: &str = "id, content, content_type, created_at, source_app, pinned, \
    deleted_at, was_compressed, original_size, compressed_size, category";

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
            category: self.category,
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

        let sql = format!(
            "INSERT INTO clips (content, content_type, source_app, was_compressed, original_size, compressed_size) \
             VALUES (?, ?, ?, ?, ?, ?) \
             RETURNING {CLIP_COLUMNS}"
        );
        let row = sqlx::query_as::<_, ClipRow>(&sql)
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
            category: row.category,
        })
    }

    pub async fn get_recent_clips(&self, limit: i64) -> Result<Vec<Clip>, DbError> {
        let sql = format!(
            "SELECT {CLIP_COLUMNS} FROM clips WHERE deleted_at IS NULL \
             ORDER BY created_at DESC LIMIT ?"
        );
        let rows = sqlx::query_as::<_, ClipRow>(&sql)
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
        let sql = format!("SELECT {CLIP_COLUMNS} FROM clips WHERE id = ?");
        let row = sqlx::query_as::<_, ClipRow>(&sql)
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

    /// Flip the `pinned` flag on a clip. Returns the new pinned state.
    pub async fn toggle_pin(&self, clip_id: i64) -> Result<bool, DbError> {
        sqlx::query("UPDATE clips SET pinned = 1 - pinned WHERE id = ?")
            .bind(clip_id)
            .execute(&self.pool)
            .await?;
        self.is_pinned(clip_id).await
    }

    /// Assign a category to a clip. Creates the category row if missing.
    /// Passing `None` clears the assignment.
    pub async fn assign_category(
        &self,
        clip_id: i64,
        category: Option<&str>,
    ) -> Result<(), DbError> {
        if let Some(name) = category {
            sqlx::query("INSERT OR IGNORE INTO categories (name) VALUES (?)")
                .bind(name)
                .execute(&self.pool)
                .await?;
        }
        sqlx::query("UPDATE clips SET category = ? WHERE id = ?")
            .bind(category)
            .bind(clip_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    /// Return all category names sorted alphabetically.
    pub async fn get_categories(&self) -> Result<Vec<String>, DbError> {
        let rows: Vec<(String,)> =
            sqlx::query_as("SELECT name FROM categories ORDER BY name ASC")
                .fetch_all(&self.pool)
                .await?;
        Ok(rows.into_iter().map(|r| r.0).collect())
    }

    /// Delete a category and unset it on any clips that referenced it.
    pub async fn delete_category(&self, name: &str) -> Result<(), DbError> {
        let mut tx = self.pool.begin().await?;
        sqlx::query("UPDATE clips SET category = NULL WHERE category = ?")
            .bind(name)
            .execute(&mut *tx)
            .await?;
        sqlx::query("DELETE FROM categories WHERE name = ?")
            .bind(name)
            .execute(&mut *tx)
            .await?;
        tx.commit().await?;
        Ok(())
    }

    /// Soft-delete a batch of clips atomically.
    pub async fn bulk_delete(&self, clip_ids: Vec<i64>) -> Result<(), DbError> {
        if clip_ids.is_empty() {
            return Ok(());
        }
        let mut tx = self.pool.begin().await?;
        for id in clip_ids {
            sqlx::query("UPDATE clips SET deleted_at = datetime('now') WHERE id = ?")
                .bind(id)
                .execute(&mut *tx)
                .await?;
        }
        tx.commit().await?;
        Ok(())
    }

    /// Soft-delete every non-pinned, non-deleted clip. Returns the row count.
    pub async fn clear_history(&self) -> Result<u64, DbError> {
        let result = sqlx::query(
            "UPDATE clips SET deleted_at = datetime('now') \
             WHERE pinned = 0 AND deleted_at IS NULL",
        )
        .execute(&self.pool)
        .await?;
        Ok(result.rows_affected())
    }

    /// Return all non-deleted clips in the given category, newest first.
    pub async fn get_clips_by_category(&self, category: &str) -> Result<Vec<Clip>, DbError> {
        let sql = format!(
            "SELECT {CLIP_COLUMNS} FROM clips \
             WHERE category = ? AND deleted_at IS NULL \
             ORDER BY created_at DESC"
        );
        let rows = sqlx::query_as::<_, ClipRow>(&sql)
            .bind(category)
            .fetch_all(&self.pool)
            .await?;
        rows.into_iter().map(|r| r.decode()).collect()
    }

    /// Return all non-deleted clips that are currently pinned, newest first.
    pub async fn get_pinned_clips(&self) -> Result<Vec<Clip>, DbError> {
        let sql = format!(
            "SELECT {CLIP_COLUMNS} FROM clips \
             WHERE pinned = 1 AND deleted_at IS NULL \
             ORDER BY created_at DESC"
        );
        let rows = sqlx::query_as::<_, ClipRow>(&sql)
            .fetch_all(&self.pool)
            .await?;
        rows.into_iter().map(|r| r.decode()).collect()
    }

    /// Return all category names as a map mirroring the categories table.
    /// Used by settings and other features that expect a HashMap view.
    #[allow(dead_code)]
    pub async fn categories_map(&self) -> Result<HashMap<String, i64>, DbError> {
        let rows: Vec<(i64, String)> =
            sqlx::query_as("SELECT id, name FROM categories")
                .fetch_all(&self.pool)
                .await?;
        Ok(rows.into_iter().map(|(id, name)| (name, id)).collect())
    }

    /// Read a single setting value by key.
    pub async fn get_setting(&self, key: &str) -> Result<Option<String>, DbError> {
        let row: Option<(String,)> = sqlx::query_as("SELECT value FROM settings WHERE key = ?")
            .bind(key)
            .fetch_optional(&self.pool)
            .await?;
        Ok(row.map(|r| r.0))
    }

    /// Upsert a single setting value.
    pub async fn set_setting(&self, key: &str, value: &str) -> Result<(), DbError> {
        sqlx::query(
            "INSERT INTO settings (key, value) VALUES (?, ?) \
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        )
        .bind(key)
        .bind(value)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// Soft-delete every non-pinned clip older than `days` days. Returns the row count.
    pub async fn expire_older_than_days(&self, days: i64) -> Result<u64, DbError> {
        let cutoff = format!("-{days} days");
        let result = sqlx::query(
            "UPDATE clips SET deleted_at = datetime('now') \
             WHERE pinned = 0 AND deleted_at IS NULL \
             AND created_at < datetime('now', ?)",
        )
        .bind(cutoff)
        .execute(&self.pool)
        .await?;
        Ok(result.rows_affected())
    }

    /// Return all settings as a key/value map.
    pub async fn get_all_settings(&self) -> Result<HashMap<String, String>, DbError> {
        let rows: Vec<(String, String)> =
            sqlx::query_as("SELECT key, value FROM settings")
                .fetch_all(&self.pool)
                .await?;
        Ok(rows.into_iter().collect())
    }

    /// Insert a clip received from a remote Synapt peer. Stored as a normal text
    /// clip tagged with `source_app = 'synapt'` and the sender's identity.
    /// Returns the new clip's rowid.
    pub async fn insert_received_clip(
        &self,
        content: &str,
        sender_peer_id: &str,
        sender_name: &str,
    ) -> Result<i64, DbError> {
        let row: (i64,) = sqlx::query_as(
            "INSERT INTO clips \
                (content, content_type, created_at, source_app, sender_name, sender_peer_id) \
             VALUES (?, 'text', datetime('now'), 'synapt', ?, ?) \
             RETURNING id",
        )
        .bind(content)
        .bind(sender_name)
        .bind(sender_peer_id)
        .fetch_one(&self.pool)
        .await?;
        Ok(row.0)
    }
}
