# JLPT N3 Learning System — Phase 3: Reading Module

**Date:** 2026-06-04
**Status:** Draft
**Version:** 1.0
**Priority:** Reading (first) → Listening (second) → Import Pipeline (third)

---

## 1. Overview

Phase 3 adds three subsystems to N3LearningOS, delivered in priority order:

1. **Reading Module** — article reader with furigana toggle, word lookup, and reading comprehension quiz
2. **Listening Module** — audio player with transcript sync and listening quiz (deferred)
3. **Import Pipeline** — user content import (CSV/JSON/APKG) (deferred)

This spec covers the full Reading module design. Listening and Import Pipeline are outlined at a high level with deferred implementation.

### Key Constraints

- Fully offline, pre-bundled content (`content/reading.json`)
- No cloud APIs, no AI features
- Follows existing island architecture pattern (vocab/grammar/kanji precedent)
- Furigana data embedded in JSON (no runtime kuromoji/wasm)
- Reuses existing SRS system (`flashcard`, `content_type='reading'`)

---

## 2. Architecture

### 2.1 Module Structure

```
src/features/reading/
  ├── pages/
  │   ├── ReadingListPage.tsx     → /reading
  │   └── ReadingDetailPage.tsx   → /reading/:id
  ├── components/
  │   ├── ReadingContent.tsx       → Main reader with ruby rendering + word lookup
  │   ├── FuriganaToggle.tsx      → Toggle switch for furigana visibility
  │   ├── WordPopover.tsx         → Popup card for looked-up words
  │   ├── QuizSection.tsx         → Comprehension questions at article bottom
  │   └── QuizQuestion.tsx        → Single question card (4 options + feedback)
  ├── hooks/
  │   └── useReading.ts           → TanStack Query hooks
  └── types/
      └── index.ts                → TypeScript types

src-tauri/src/
  ├── commands/
  │   └── reading.rs              → Rust commands for reading module
  ├── importers/
  │   └── reading.rs              → Reading content importer (manifest-driven)
  └── migrations/
      └── 003_reading.sql         → Reading table migration
```

### 2.2 Communication

- Frontend calls `invoke("list_reading")`, `invoke("get_reading_detail")`, `invoke("lookup_word")`
- No new Tauri events needed
- Content import runs through existing manifest-driven pipeline

### 2.3 Routing

```
/reading       → ReadingListPage (article list with search)
/reading/:id   → ReadingDetailPage (reader + quiz)
```

---

## 3. Data Design

### 3.1 Database Schema (Migration 003)

```sql
CREATE TABLE IF NOT EXISTS reading (
    id          TEXT PRIMARY KEY,
    title       TEXT NOT NULL,
    segments    TEXT NOT NULL,         -- JSON array of Segment
    questions   TEXT,                  -- JSON array of Question (nullable)
    source      TEXT NOT NULL DEFAULT 'bundled',
    level       TEXT NOT NULL DEFAULT 'N3',
    is_bookmarked INTEGER DEFAULT 0,
    created_at  TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_reading_title ON reading(title);
```

### 3.2 Content Format (`content/reading.json`)

```json
{
  "version": "1.0.0",
  "items": [
    {
      "id": "reading_001",
      "title": "日本の電車文化",
      "level": "N3",
      "segments": [
        [
          { "t": "日本", "f": "にほん", "k": true },
          { "t": "の", "k": false },
          { "t": "電車", "f": "でんしゃ", "k": true },
          { "t": "は、", "k": false }
        ],
        [
          { "t": "時間", "f": "じかん", "k": true },
          { "t": "に", "k": false },
          { "t": "正確", "f": "せいかく", "k": true },
          { "t": "で有名です。", "k": false }
        ]
      ],
      "questions": [
        {
          "id": "q1",
          "question": "日本の電車の特徴は何ですか？",
          "options": ["安いこと", "時間に正確なこと", "速いこと", "新しいこと"],
          "correctIndex": 1,
          "explanation": "本文で「時間に正確で有名です」と述べられています。"
        }
      ],
      "source": "bundled"
    }
  ]
}
```

**Segment format:**
- `t`: text string
- `f`: furigana reading (omitted/null when `k` is false)
- `k`: boolean — `true` = kanji compound with furigana, `false` = kana/punctuation (no ruby needed)

Segments are grouped into paragraphs (array of arrays). Each paragraph renders as a `<p>` block.

### 3.3 Rust Models (add to `models/mod.rs`)

```rust
#[derive(Debug, Serialize, Deserialize, Clone, sqlx::FromRow)]
pub struct Reading {
    pub id: String,
    pub title: String,
    pub segments: String,        // JSON stored as text
    pub questions: Option<String>,
    pub source: String,
    pub level: String,
    pub is_bookmarked: bool,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Segment {
    pub t: String,               // text
    #[serde(skip_serializing_if = "Option::is_none")]
    pub f: Option<String>,       // furigana
    pub k: bool,                 // is kanji compound
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Question {
    pub id: String,
    pub question: String,
    pub options: Vec<String>,
    pub correct_index: u8,
    pub explanation: String,
}
```

---

## 4. Rust Commands

### `list_reading`

- **Path:** `commands/reading.rs`
- **Method:** `invoke("list_reading", { page, pageSize, query })`
- **Returns:** `PaginatedResult<Reading>`
- **Query:** `LIKE '%query%'` on `title` (existing pattern)
- **Defaults:** page=1, page_size=20

### `get_reading_detail`

- **Method:** `invoke("get_reading_detail", { id })`
- **Returns:** `Reading` with parsed segments/questions
- **Edge case:** Returns error `"Reading not found"` if id doesn't exist

### `lookup_word`

- **Method:** `invoke("lookup_word", { word })`
- **Returns:** `Option<Vocabulary>` — first match in vocabulary table by word
- **Edge case:** Returns null if no match found (show "word not found" state)
- **Note:** Case-insensitive match on `word` column

---

## 5. Frontend Component Design

### 5.1 ReadingListPage

- **Loading state:** Skeleton list (4 x card shape)
- **Error state:** Error toast with retry button
- **Empty state:** "暂无阅读文章" illustration + message
- **Success state:** Search bar + card grid
- Each card: title, level badge (`N3`), bookmark icon
- Search triggers `list_reading` with query parameter

### 5.2 ReadingDetailPage

- **Loading state:** Full-page skeleton (title placeholder + paragraph skeletons)
- **Error state:** Error toast with "返回列表" link
- **Not-found state:** "文章不存在" centered message
- **Success state:** Reader layout:

```
┌─────────────────────────────────────┐
│ ← 返回列表   标题            🔖 ☆   │
├─────────────────────────────────────┤
│                                     │
│  [振り仮名: 表示 | 非表示]          │
│                                     │
│  ┌─────────────────────────────┐   │
│  │ 段落1 — 文字                 │   │
│  │ 段落2 — 文字                 │   │
│  │ 段落3 — 文字                 │   │
│  └─────────────────────────────┘   │
│                                     │
│  ─── 読解問題 ───                   │
│                                     │
│  問題1: 文章の内容と合っているもの   │
│  はどれですか？                     │
│                                     │
│  ○ 選択肢A                         │
│  ○ 選択肢B                         │
│  ○ 選択肢C                         │
│  ○ 選択肢D                         │
│                                     │
│  [结果反馈 + 解析]                  │
│                                     │
│  問題2: ...                         │
│                                     │
└─────────────────────────────────────┘
```

### 5.3 Furigana Rendering (ReadingContent component)

- Each `Segment` with `k=true` renders as `<ruby>漢字<rt>よみ</rt></ruby>`
- Each `Segment` with `k=false` renders as plain text
- Toggle: CSS class `.hide-furigana rt { display: none; }`
- Click handler on kanji segments: check if a vocabulary word starts with this text
- State machine: `normal → popover (word found) | normal → noop (no match)`

### 5.4 WordPopover

- Triggered by clicking a kanji segment
- Calls `lookup_word` with the segment text
- States:
  - **Loading:** Spinner inside popover
  - **Found:** Word card showing word/reading/meaning/example
  - **Not found:** "未在词汇库中找到" message
- Dismiss: click outside or ESC

### 5.5 QuizSection

- Rendered below article content
- Each question is a standalone card
- User selects one option → immediately shows correct/incorrect + explanation
- Visual: green border for correct, red border for wrong + highlight correct answer
- All questions visible at once, answered independently
- Results summary: "答对 X/Y 题" at bottom

---

## 6. Content Package

### 6.1 migration 003

New file: `src-tauri/migrations/003_reading.sql`

### 6.2 Content JSON

New file: `content/reading.json` — 15 N3-level reading articles covering common JLPT topics (culture, daily life, technology, environment, health, etc.)

Each article has 10+ paragraphs and 2-3 comprehension questions.

### 6.3 Manifest Update

`content/manifest.json` — add new entry:
```json
{
  "type": "reading",
  "file": "reading.json",
  "version": "1.0.0"
}
```

### 6.4 Importer

New file: `src-tauri/src/importers/reading.rs`

- `import_reading(content_dir)` — parse reading.json, insert into reading table, update metadata version
- Flashcard creation: reading articles create flashcards too (`content_type='reading'`), so they appear in the review queue for FSRS scheduling
- Register in `importers/mod.rs`

---

## 7. Flashcard Integration

Reading articles create flashcards like other content types:

- **Content type:** `'reading'`
- **Review front:** Article title
- **Review back:** Asks user if they understood the article (subjective self-rating)
- **Rating:** Again/Hard/Good/Easy (same FSRS system)
- Actually, debating this: reading articles are consumed, not reviewed via flashcards in the traditional sense. The comprehension quiz at the bottom serves as the assessment.

**Decision:** Do NOT create flashcards for reading articles. Comprehension is assessed via inline quiz questions. This avoids polluting the SRS queue with "did you read this?" cards that don't meaningfully test recall.

---

## 8. Deferred: Listening Module (Outline)

**To be spec'd when reading is complete.**

- Audio file pre-bundled in `content/audio/` directory
- Rust backend: audio playback via Tauri `webkit2gtk` or `rodio` crate
- Transcript with timestamps (JSON segments)
- Sync: highlight current sentence during playback
- Speed control: 0.75x, 1x, 1.25x, 1.5x
- Listening comprehension quiz (same format as reading questions)
- Database table `listening` with similar structure to reading

## 9. Deferred: Import Pipeline (Outline)

**To be spec'd when reading + listening are complete.**

- CSV importer: user selects file → maps columns → imports to vocabulary table
- JSON importer: flexible schema mapping
- APKG importer: parses Anki package format → extracts cards → maps to content format
- Existing `importers/` module expansion

---

## 10. Error Handling

| Scenario | Frontend Behavior | Backend Response |
|---|---|---|
| Article ID not found | "文章不存在" + redirect to list | `Err("Reading not found")` |
| Word lookup no match | "未在词汇库中找到" in popover | `Ok(None)` |
| Empty reading list | "暂无阅读文章" illustration | Empty `PaginatedResult` |
| Search no results | "没有找到相关文章" | Empty `PaginatedResult` |
| DB connection fail | Error toast + retry | `Err(database error)` |
| Network (offline) | Tauri invoke fails → error toast | N/A (local only) |

---

## 11. Testing

### Backend (Rust)

- `test_import_reading` — import reading.json, verify rows in DB
- `test_list_reading_pagination` — verify pagination
- `test_lookup_word_found` — lookup existing word
- `test_lookup_word_not_found` — lookup missing word

### Frontend

- `FuriganaToggle` — toggle shows/hides `<rt>` elements
- `WordPopover` — API call on click, displays result
- `QuizSection` — select correct/incorrect, verify feedback color and explanation

---

## 12. File Change Summary

| File | Action |
|---|---|
| `src-tauri/migrations/003_reading.sql` | Create |
| `src-tauri/src/models/mod.rs` | Add Reading, Segment, Question structs |
| `src-tauri/src/commands/reading.rs` | Create |
| `src-tauri/src/commands/mod.rs` | Add `pub mod reading;` |
| `src-tauri/src/importers/reading.rs` | Create |
| `src-tauri/src/importers/mod.rs` | Add reading module to pipeline |
| `src-tauri/src/lib.rs` | Register reading commands, update import call |
| `content/reading.json` | Create (15 articles) |
| `content/manifest.json` | Add reading entry |
| `src/features/reading/pages/ReadingListPage.tsx` | Create |
| `src/features/reading/pages/ReadingDetailPage.tsx` | Create |
| `src/features/reading/components/ReadingContent.tsx` | Create |
| `src/features/reading/components/FuriganaToggle.tsx` | Create |
| `src/features/reading/components/WordPopover.tsx` | Create |
| `src/features/reading/components/QuizSection.tsx` | Create |
| `src/features/reading/components/QuizQuestion.tsx` | Create |
| `src/features/reading/hooks/useReading.ts` | Create |
| `src/features/reading/types/index.ts` | Create |
| `src/routes/index.tsx` | Add reading routes |
| `src/services/api.ts` | Add invoke wrappers |
| `src/features/dashboard/hooks/useDashboard.ts` | Optional: update progress for reading |
