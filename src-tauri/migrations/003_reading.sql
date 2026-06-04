CREATE TABLE IF NOT EXISTS reading (
    id          TEXT PRIMARY KEY,
    title       TEXT NOT NULL,
    segments    TEXT NOT NULL,         -- JSON array of paragraph arrays
    questions   TEXT,                  -- JSON array of Question (nullable)
    source      TEXT NOT NULL DEFAULT 'bundled',
    level       TEXT NOT NULL DEFAULT 'N3',
    is_bookmarked INTEGER DEFAULT 0,
    created_at  TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_reading_title ON reading(title);
