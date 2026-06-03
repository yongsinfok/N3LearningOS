CREATE TABLE IF NOT EXISTS grammar (
    id          TEXT PRIMARY KEY,
    pattern     TEXT NOT NULL,
    meaning     TEXT NOT NULL,
    explanation TEXT NOT NULL,
    examples    TEXT NOT NULL,         -- JSON array
    related     TEXT,                  -- Related grammar IDs (JSON array)
    source      TEXT NOT NULL DEFAULT 'bundled',
    created_at  TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS kanji (
    id          TEXT PRIMARY KEY,
    character   TEXT NOT NULL,
    onyomi      TEXT,                  -- JSON array
    kunyomi     TEXT,                  -- JSON array
    meaning     TEXT NOT NULL,
    stroke_svg  TEXT,                  -- Path to SVG
    example_words TEXT,                -- JSON array of compound words
    source      TEXT NOT NULL DEFAULT 'bundled',
    created_at  TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS note (
    id           TEXT PRIMARY KEY,
    content_id   TEXT NOT NULL,
    content_type TEXT NOT NULL,
    content      TEXT NOT NULL,
    created_at   TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at   TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_note_content ON note(content_id, content_type);
CREATE INDEX IF NOT EXISTS idx_grammar_pattern ON grammar(pattern);
CREATE INDEX IF NOT EXISTS idx_kanji_character ON kanji(character);
