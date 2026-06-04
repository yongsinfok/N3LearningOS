CREATE TABLE IF NOT EXISTS listening (
    id          TEXT PRIMARY KEY,
    title       TEXT NOT NULL,
    audio_file  TEXT NOT NULL,
    transcript  TEXT NOT NULL,
    questions   TEXT,
    source      TEXT NOT NULL DEFAULT 'bundled',
    level       TEXT NOT NULL DEFAULT 'N3',
    is_bookmarked INTEGER DEFAULT 0,
    created_at  TEXT NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX IF NOT EXISTS idx_listening_title ON listening(title);
