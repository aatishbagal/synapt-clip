CREATE TABLE IF NOT EXISTS clips (
    id           INTEGER PRIMARY KEY AUTOINCREMENT,
    content      TEXT NOT NULL,
    content_type TEXT NOT NULL DEFAULT 'text',
    created_at   TEXT NOT NULL DEFAULT (datetime('now')),
    source_app   TEXT,
    pinned       INTEGER NOT NULL DEFAULT 0,
    deleted_at   TEXT
);

CREATE INDEX IF NOT EXISTS idx_clips_created ON clips(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_clips_pinned ON clips(pinned);
