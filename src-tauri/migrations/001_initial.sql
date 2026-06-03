CREATE TABLE IF NOT EXISTS vocabulary (
    id          TEXT PRIMARY KEY,
    word        TEXT NOT NULL,
    reading     TEXT NOT NULL,
    meaning     TEXT NOT NULL,
    example     TEXT,
    tags        TEXT,
    source      TEXT NOT NULL DEFAULT 'bundled',
    created_at  TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS flashcard (
    id              TEXT PRIMARY KEY,
    content_id      TEXT NOT NULL,
    content_type    TEXT NOT NULL,
    due_date        TEXT NOT NULL,
    stability       REAL NOT NULL DEFAULT 0,
    difficulty      REAL NOT NULL DEFAULT 0,
    elapsed_days    INTEGER NOT NULL DEFAULT 0,
    scheduled_days  INTEGER NOT NULL DEFAULT 0,
    reps            INTEGER NOT NULL DEFAULT 0,
    lapses          INTEGER NOT NULL DEFAULT 0,
    state           INTEGER NOT NULL DEFAULT 0,
    last_review     TEXT,
    created_at      TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS review_history (
    id              TEXT PRIMARY KEY,
    flashcard_id    TEXT NOT NULL,
    review_date     TEXT NOT NULL,
    rating          INTEGER NOT NULL,
    elapsed_secs    INTEGER,
    created_at      TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS study_session (
    id              TEXT PRIMARY KEY,
    module_type     TEXT,
    start_time      TEXT NOT NULL,
    end_time        TEXT,
    cards_reviewed  INTEGER DEFAULT 0,
    correct_count   INTEGER DEFAULT 0
);

CREATE TABLE IF NOT EXISTS bookmark (
    id           TEXT PRIMARY KEY,
    content_id   TEXT NOT NULL,
    content_type TEXT NOT NULL,
    note         TEXT,
    created_at   TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(content_id, content_type)
);

CREATE TABLE IF NOT EXISTS daily_stats (
    date            TEXT PRIMARY KEY,
    study_minutes   INTEGER DEFAULT 0,
    cards_reviewed  INTEGER DEFAULT 0,
    correct_count   INTEGER DEFAULT 0,
    vocabulary_new  INTEGER DEFAULT 0
);

CREATE TABLE IF NOT EXISTS streak_stats (
    id              INTEGER PRIMARY KEY DEFAULT 1,
    current_streak  INTEGER DEFAULT 0,
    longest_streak  INTEGER DEFAULT 0,
    last_study_date TEXT
);

CREATE TABLE IF NOT EXISTS metadata (
    key   TEXT PRIMARY KEY,
    value TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_flashcard_due ON flashcard(due_date);
CREATE INDEX IF NOT EXISTS idx_flashcard_content ON flashcard(content_id, content_type);
CREATE INDEX IF NOT EXISTS idx_review_history_flashcard ON review_history(flashcard_id);
CREATE INDEX IF NOT EXISTS idx_review_history_date ON review_history(review_date);
