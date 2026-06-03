# JLPT N3 Learning System — Design Specification

**Date:** 2026-06-03
**Status:** Draft
**Version:** 1.0

---

## 1. Overview

Fully offline, local-first JLPT N3 preparation desktop app. Built with Tauri v2 (Rust backend, React/TypeScript frontend), SQLite storage, FSRS-5 spaced repetition. Design inspired by Duolingo — gamified, habit-driven daily flow.

**Key constraints:**
- Zero cloud dependencies
- Pre-bundled content (resource directory alongside binary)
- Single-user, no accounts
- No AI features

---

## 2. Architecture

### 2.1 Application Framework

```
┌───────────────────────────────────────────────┐
│              Tauri v2 Shell                     │
│                                                  │
│  ┌────────────────────┐  ┌──────────────────┐  │
│  │   React Frontend    │  │  Rust Backend     │  │
│  │ (Vite + TS + React) │  │ (Tauri Commands)  │  │
│  │                     │  │                   │  │
│  │ ┌─────────────────┐│  │ ┌─────────────┐  │  │
│  │ │ Dashboard Hub    ││  │ │ FSRS-5 SRS  │  │  │
│  │ │ (Daily Flow)     ││  │ │ Engine      │  │  │
│  │ ├─────────────────┤│  │ ├─────────────┤  │  │
│  │ │ Module Pages     ││  │ │ Importers   │  │  │
│  │ │ (vocab/grammar/  │◄─┼─┼─┤ (JSON/CSV/  │  │  │
│  │ │  kanji/reading/  ││  │ │  APKG/manifest)│  │
│  │ │  listening/exams)││  │ ├─────────────┤  │  │
│  │ └─────────────────┘│  │ │ SQLx ORM    │  │  │
│  │                     │  │ ├─────────────┤  │  │
│  │ Zustand (state)     │  │ │ SQLite DB   │  │  │
│  │ TanStack Query (svc)│  │ └─────────────┘  │  │
│  └────────────────────┘  └──────────────────┘  │
└───────────────────────────────────────────────┘
```

### 2.2 Communication

- **Tauri Commands:** Frontend calls `invoke()` for all data operations. No HTTP API.
- **Events:** Tauri event system for background notifications (import progress, etc.).
- All data operations go through Rust — frontend never accesses SQLite directly.

### 2.3 State Management

**Global Zustand stores:**
- User settings (theme, daily goal, playback speed, SRS config)
- Dashboard state (streak, due count, XP/level)
- Gamification state (achievements, badges)

**Per-module Zustand stores:**
- Current view (list, detail, quiz mode)
- Filters / search state
- Pagination / loading state

**Server state (TanStack Query):**
- All data fetched via Tauri `invoke()`
- Cache + background refetch + optimistic updates

### 2.4 Module Structure Convention

Every feature module follows the same structure:

```
features/vocabulary/
  ├── pages/        → Route entry points (ListPage, DetailPage, QuizPage)
  ├── components/   → Module-specific components
  ├── hooks/        → Module-specific React hooks
  ├── store/        → Module Zustand store
  ├── types/        → TypeScript types
  └── index.ts      → Public exports

src-tauri/src/commands/
  └── vocabulary.rs → Corresponding Rust commands
```

### 2.5 Routing

Flat routes, max 2 levels deep:

```
/              → Dashboard (default landing)
/vocabulary    → Vocabulary list
/vocabulary/:id → Vocabulary detail
/vocabulary/quiz → Quiz
/grammar       → Grammar list
/grammar/:id   → Grammar detail
/kanji         → Kanji list
/kanji/:id     → Kanji detail
/reading       → Reading list
/reading/:id   → Reader
/listening     → Listening list
/listening/:id → Player
/exams         → Exam list/history
/exams/new     → Exam setup
/exams/:id     → Exam in progress / results
/settings      → Settings
```

---

## 3. Database Design

### 3.1 Schema

```sql
-- Content tables
CREATE TABLE vocabulary (
    id          TEXT PRIMARY KEY,
    word        TEXT NOT NULL,
    reading     TEXT NOT NULL,
    meaning     TEXT NOT NULL,
    example     TEXT,
    tags        TEXT,                  -- JSON array
    source      TEXT NOT NULL,
    created_at  TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE grammar (
    id          TEXT PRIMARY KEY,
    pattern     TEXT NOT NULL,
    meaning     TEXT NOT NULL,
    explanation TEXT NOT NULL,
    examples    TEXT NOT NULL,         -- JSON array
    related     TEXT,                  -- Related grammar IDs (JSON array)
    source      TEXT NOT NULL,
    created_at  TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE kanji (
    id          TEXT PRIMARY KEY,
    character   TEXT NOT NULL,
    onyomi      TEXT,                  -- JSON array
    kunyomi     TEXT,                  -- JSON array
    meaning     TEXT NOT NULL,
    stroke_svg  TEXT,                  -- Path to SVG file in resources
    source      TEXT NOT NULL,
    created_at  TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE reading (
    id          TEXT PRIMARY KEY,
    title       TEXT NOT NULL,
    content     TEXT NOT NULL,
    furigana    TEXT,                  -- JSON mapping {"漢字": "かんじ"}
    article_url TEXT,                  -- Original URL
    source      TEXT NOT NULL,
    is_bookmarked INTEGER DEFAULT 0,
    created_at  TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE listening (
    id          TEXT PRIMARY KEY,
    title       TEXT NOT NULL,
    audio_path  TEXT NOT NULL,         -- Path to local audio file
    transcript  TEXT NOT NULL,         -- Full transcript
    timestamps  TEXT,                  -- JSON: [{"time": 0, "text": "..."}]
    duration    INTEGER,               -- Duration in seconds
    source      TEXT NOT NULL,
    created_at  TEXT NOT NULL DEFAULT (datetime('now'))
);

-- SRS / Learning tracking
CREATE TABLE flashcard (
    id              TEXT PRIMARY KEY,
    content_id      TEXT NOT NULL,
    content_type    TEXT NOT NULL,     -- 'vocabulary' | 'grammar' | 'kanji'
    due_date        TEXT NOT NULL,
    stability       REAL NOT NULL DEFAULT 0,
    difficulty      REAL NOT NULL DEFAULT 0,
    elapsed_days    INTEGER NOT NULL DEFAULT 0,
    scheduled_days  INTEGER NOT NULL DEFAULT 0,
    reps            INTEGER NOT NULL DEFAULT 0,
    lapses          INTEGER NOT NULL DEFAULT 0,
    state           INTEGER NOT NULL DEFAULT 0, -- 0=new, 1=learning, 2=review, 3=relearning
    last_review     TEXT,
    created_at      TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE review_history (
    id              TEXT PRIMARY KEY,
    flashcard_id    TEXT NOT NULL,
    review_date     TEXT NOT NULL,
    rating          INTEGER NOT NULL,  -- 0=Again, 1=Hard, 2=Good, 3=Easy
    elapsed_secs    INTEGER,
    created_at      TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE study_session (
    id              TEXT PRIMARY KEY,
    module_type     TEXT,              -- NULL = comprehensive
    start_time      TEXT NOT NULL,
    end_time        TEXT,
    cards_reviewed  INTEGER DEFAULT 0,
    correct_count   INTEGER DEFAULT 0
);

-- User data
CREATE TABLE bookmark (
    id           TEXT PRIMARY KEY,
    content_id   TEXT NOT NULL,
    content_type TEXT NOT NULL,
    note         TEXT,
    created_at   TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(content_id, content_type)
);

CREATE TABLE note (
    id           TEXT PRIMARY KEY,
    content_id   TEXT NOT NULL,
    content_type TEXT NOT NULL,
    content      TEXT NOT NULL,
    created_at   TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at   TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Statistics
CREATE TABLE daily_stats (
    date            TEXT PRIMARY KEY,  -- 'YYYY-MM-DD'
    study_minutes   INTEGER DEFAULT 0,
    cards_reviewed  INTEGER DEFAULT 0,
    correct_count   INTEGER DEFAULT 0,
    vocabulary_new  INTEGER DEFAULT 0,
    grammar_new     INTEGER DEFAULT 0
);

CREATE TABLE streak_stats (
    id              INTEGER PRIMARY KEY DEFAULT 1,  -- Single row
    current_streak  INTEGER DEFAULT 0,
    longest_streak  INTEGER DEFAULT 0,
    last_study_date TEXT
);

-- Indexes
CREATE INDEX idx_flashcard_due ON flashcard(due_date);
CREATE INDEX idx_flashcard_content ON flashcard(content_id, content_type);
CREATE INDEX idx_review_history_date ON review_history(review_date);
CREATE INDEX idx_review_history_card ON review_history(flashcard_id);
```

### 3.2 Design Decisions

| Decision | Rationale |
|---|---|
| `content_id` + `content_type` polymorphism | Single flashcard/review table, one SRS engine for all modules |
| JSON fields for arrays | SQLite has native JSON functions; avoids explosion of join tables for tags/examples |
| Single-row `streak_stats` | Avoids aggregation query on every dashboard load |
| `source` on every content row | Required by PRD for source attribution |
| Application-level referential integrity | Shuns SQLite foreign key performance hit |

---

## 4. SRS Engine (FSRS-5)

### 4.1 Algorithm

- **Primary:** FSRS-5 (Free Spaced Repetition Scheduler)
- **Fallback:** SM-2 (when FSRS parameters unavailable or corrupted)
- **Storage:** `~/.n3-learning/fsrs_params.json`

### 4.2 Review Flow

```
Front of card shown → User thinks → "Show answer" tapped
    → Back of card shown + 4 rating buttons [Again] [Hard] [Good] [Easy]
    → User taps rating → Frontend invoke('review_card', {card_id, rating})
    → Rust FSRS engine:
        1. Read current stability/difficulty/state
        2. Compute new interval from rating
        3. Update flashcard row
        4. Insert review_history row
        5. Return new due_date
    → Frontend updates UI, advance to next card
```

### 4.3 Rating Buttons

| Button | Rating | Meaning |
|---|---|---|
| Again | 0 | Complete forget / wrong answer |
| Hard | 1 | Recalled with difficulty |
| Good | 2 | Normal recall |
| Easy | 3 | Effortless recall |

### 4.4 Rust Module Structure

```
src-tauri/src/fsrs/
  ├── mod.rs        → Public API: review_card(), get_due_cards(), get_dashboard()
  ├── fsrs.rs       → FSRS-5 implementation
  ├── sm2.rs        → SM-2 fallback
  ├── params.rs     → Parameter load/save
  └── types.rs      → Card, ReviewLog, Rating types
```

### 4.5 Frontend Components

```
shared/components/review/
  ├── Flashcard.tsx        → Generic card front/back
  ├── ReviewRating.tsx     → [Again] [Hard] [Good] [Easy] button group
  ├── ReviewQueue.tsx      → Queue management + progress bar
  └── SessionSummary.tsx   → Post-review summary panel
```

---

## 5. Dashboard & Daily Flow

### 5.1 Default Landing Page

Dashboard is the default view. It shows:
- Greeting + streak fire icon
- Due card count + new card count
- "Start Learning" CTA button
- Module progress bars (vocabulary %, grammar %, kanji %)
- Weekly activity mini-chart
- Quick navigation to each module

### 5.2 Mixed Study Flow

"Start Learning" → pulls from all overdue flashcards:

| Content type | Proportion |
|---|---|
| Vocabulary | ~50% |
| Grammar | ~30% |
| Kanji | ~20% |

After last card → `SessionSummary` (correct %, time spent, XP earned) → back to dashboard with updated progress.

### 5.3 Dashboard Data

Frontend calls `invoke('get_dashboard')` → Rust aggregates:

```rust
struct DashboardData {
    streak: i32,
    due_count: i64,
    new_available: i64,
    module_progress: Vec<ModuleProgress>,
    weekly_activity: Vec<DailyActivity>,
    weak_areas: Vec<WeakArea>,
}
```

---

## 6. Module Specifications

### 6.1 Vocabulary

**Routes:** `/vocabulary`, `/vocabulary/:id`, `/vocabulary/quiz`

**List page:** Search bar, tag filter, sortable rows (word | reading | meaning | mastery %)

**Detail page:** Flashcard-style presentation. Front shows kanji/word, back shows reading + meaning + example sentence + tags + bookmark/note actions.

**Quiz modes:**
- Japanese → English (4-choice or input)
- English → Japanese (4-choice or input)
- Reading selection (word → choose reading)
- Fill in the blank (sentence with gap)
- Multiple choice

**Rust commands:**
```rust
#[tauri::command]
fn list_vocabulary(page: u32, search: Option<String>, tag_filter: Option<Vec<String>>) -> Result<PaginatedVocab>

#[tauri::command]
fn get_vocabulary_detail(id: String) -> Result<VocabDetail>

#[tauri::command]
fn start_vocab_quiz(mode: QuizMode, count: u32) -> Result<Vec<QuizQuestion>>

#[tauri::command]
fn submit_vocab_answer(quiz_id: String, answer: String) -> Result<QuizResult>
```

### 6.2 Grammar

**Routes:** `/grammar`, `/grammar/:id`

**List page:** Grouped by function (reason/cause, condition, obligation, etc.). Each row shows pattern + meaning + mastery.

**Detail page:** Pattern display, detailed explanation, conjugation rules, example sentences (2-3), related grammar links. Bookmark + note + practice actions.

**Quiz:** Context-based grammar selection, fill-in-the-blank,辨析 (discrimination between similar patterns).

### 6.3 Kanji

**Routes:** `/kanji`, `/kanji/:id`

**List page:** Grid layout (character | onyomi | kunyomi | mastery). Filters: JLPT level, radical, stroke count.

**Detail page:** Large character display, onyomi/kunyomi lists, meaning, stroke order SVG animation, example words (compounds using this kanji).

**Writing canvas:** HTML5 Canvas with reference character overlay. No OCR — purely for muscle memory practice. User draws, self-judges.

**Quiz:** Kanji → reading, reading → kanji, meaning → kanji.

### 6.4 Reading

**Routes:** `/reading`, `/reading/:id`, `/reading/:id/quiz`

**Source:** NHK Easy News articles bundled as JSON with pre-parsed furigana mappings.

**Reader:**
- Furigana toggle (on = `<ruby>` rendering, off = plain text)
- Tap word → popup with reading + meaning + "add to review" button
- Bookmark article

**Furigana data:** Pre-computed at import time, stored as JSON mapping in `reading.furigana`.

**Reading quiz:** 3-5 comprehension questions per article (content understanding, vocabulary in context, grammar in context). Answers reference the relevant passage.

### 6.5 Listening

**Source:** Pre-bundled MP3 audio files + JSON transcript with timestamps.

**Player:**
- Waveform visualization (optional)
- Play/pause, skip forward/back, sentence-loop toggle
- Playback speed: 0.75x, 1x, 1.25x, 1.5x
- Synchronized transcript — current sentence highlighted
- Click sentence → jump to that position
- Bookmark lesson

**Speed change:** Handled in Rust backend — decode audio with `symphonia`, resample with `libresample`, return PCM buffer to frontend `AudioContext`.

**Quiz:** Listen to sentence → choose meaning, listen to dialog → answer questions.

### 6.6 Mock Exams

**Routes:** `/exams`, `/exams/new`, `/exams/:id`

**Exam types:**

| Type | Content | Questions | Duration |
|---|---|---|---|
| Vocabulary (文字・語彙) | Vocabulary MC | ~20 | 15 min |
| Grammar (文法) | Grammar MC | ~20 | 15 min |
| Reading (読解) | Passage comprehension | ~15 | 25 min |
| Listening (聴解) | Audio MC | ~15 | 20 min |
| Mixed (総合) | All sections combined | ~70 | 75 min |

**Flow:** Setup → confirm → timer start → one question at a time → auto-grade on completion → results page.

**Results page:** Score per section, correct/incorrect list with review, wrong answers auto-pushed to SRS queue as Again.

**Timer:** Countdown mode (simulated exam) or count-up (practice). Auto-submit on timeout. 5-minute warning sound.

**Rust data types:**
```rust
struct MockExam {
    id: String,
    exam_type: ExamType,
    questions: Vec<Question>,
    time_limit_secs: i32,
    created_at: String,
}

struct ExamResult {
    exam_id: String,
    scores: HashMap<ExamSection, Score>,
    total_score: i32,
    max_score: i32,
    time_spent_secs: i32,
    answers: Vec<UserAnswer>,
    completed_at: String,
}
```

---

## 7. Data Import Pipeline

### 7.1 Pre-bundled Content

```
content/                    ← Alongside executable
  ├── manifest.json         ← Version + file manifest with checksums
  ├── vocabulary.json       ← Vocabulary data
  ├── grammar.json          ← Grammar data
  ├── kanji.json            ← Kanji data
  ├── readings/
  │   ├── article_001.json
  │   └── article_002.json
  └── listening/
      ├── lesson_01.mp3
      └── lesson_01.json    ← Transcript + timestamps
```

**First-run flow:** Read `manifest.json` → compare version against DB `metadata` table → if changed, truncate content type and re-import → mark version → subsequent launches skip.

### 7.2 Importer Modules

```
src-tauri/src/importers/
  ├── mod.rs            → trait Importer { fn import() -> Result }
  ├── vocabulary.rs     → JSON → vocabulary table
  ├── grammar.rs        → JSON → grammar table
  ├── kanji.rs          → JSON → kanji table
  ├── reading.rs        → JSON → reading table
  ├── listening.rs      → JSON + audio paths → listening table
  └── manifest.rs       → Manifest validation + version tracking
```

### 7.3 User Import (Settings)

Secondary feature for user-provided content:

- **CSV import:** Column mapping wizard (word/reading/meaning)
- **JSON import:** Auto-detect field names
- **APKG import:** Extract and parse Anki SQLite database

User imports add to a separate content scope (not pre-bundled). Implemented as Tauri commands with independent import paths.

---

## 8. UI/UX

### 8.1 Design Inspiration

Duolingo-inspired:
- Playful, colorful but not childish
- Progress bars and progress circles as visual feedback
- Streak fire icon as motivation anchor
- XP points and level system (Phase 4)
- Smooth micro-animations (card flip, progress fill, streak fire)
- Warm accent colors (green success, orange streak, blue primary)

### 8.2 UI System

- **Component library:** shadcn/ui (Radix primitives + TailwindCSS)
- **Theme:** Dark + Light mode via TailwindCSS `dark:` variant
- **Layout:** Sidebar or top-bar navigation + main content area
- **States:** Loading (skeleton), empty (illustration + message), error (retry button), success (animation)

### 8.3 Accessibility

- Keyboard navigation for all quiz/review actions
- ARIA labels on interactive elements
- Focus management during card reviews and quizzes
- Sufficient color contrast in both themes

---

## 9. Gamification (Phase 4+)

| Feature | Detail |
|---|---|
| XP per review | Correct answer → XP bonus based on rating (Good/Easy more) |
| Levels | Every N XP → level up with animation |
| Streak | Consecutive days of study. Breaks reset. |
| Achievements | Milestones: 100 reviews, 500 reviews, 100% vocab, etc. |

---

## 10. Testing Strategy

| Layer | Approach |
|---|---|
| Rust unit tests | FSRS algorithm correctness, importer parsing, DB queries |
| Rust integration tests | Full command flow with in-memory SQLite |
| React component tests | Vitest + testing-library for UI logic |
| E2E | Manual testing (single-user desktop app, no CI needed) |

---

## 11. Phase Delivery Plan

### Phase 1 — MVP (Weeks 1-4)

**Goal:** Usable daily study loop. User learns vocabulary, reviews, sees progress.

- Tauri v2 project scaffold (Rust + React + Vite + Tailwind + shadcn/ui)
- SQLite schema + SQLx migrations
- FSRS-5 engine (Rust) + SM-2 fallback
- Vocabulary module CRUD (Rust commands)
- Flashcard review UI (front → back → rate)
- Dashboard MVP (due count, streak, progress bars, start button)
- Pre-bundled content import (vocabulary.json via manifest)
- Dark/light mode toggle

**Deliverable:** User opens app → sees dashboard → clicks "Start Learning" → reviews vocabulary flashcards → sees progress update.

### Phase 2 — Grammar + Kanji + Depth (Weeks 5-8)

- Grammar module (list, detail, quiz)
- Kanji module (list, detail, stroke order SVG, writing canvas)
- Grammar/Kanji SRS integration
- Global search across all modules
- Notes system (per-content CRUD)
- UI polish (animations, transitions, loading/empty/error states)
- Grammar + Kanji content packages

### Phase 3 — Reading + Listening + Import (Weeks 9-12)

- Reader with furigana toggle + vocabulary popup
- Reading comprehension quiz
- Audio player (Rust decode + speed change, synchronized transcript, sentence loop)
- Listening quiz
- CSV/JSON/APKG user import
- Reading + listening content packages

### Phase 4 — Exams + Analytics (Weeks 13-16)

- Mock exam engine (timer, auto-grade)
- 5 exam types (vocab/grammar/reading/listening/mixed)
- Results review + wrong answer SRS queue
- Exam history + performance trends
- XP/level/achievements gamification
- Detailed daily/weekly/monthly stats

### Phase 5 — N2/N1 Expansion (Future)

- N2 content packages (all types)
- N1 content packages
- Level switcher in settings
- Cross-level mixed review

---

## 12. File Structure

```
jlpt-n3-app/
├── src/
│   ├── features/
│   │   ├── dashboard/
│   │   ├── vocabulary/
│   │   ├── grammar/
│   │   ├── kanji/
│   │   ├── reading/
│   │   ├── listening/
│   │   ├── exams/
│   │   └── settings/
│   ├── components/       → Shared UI components
│   ├── hooks/            → Shared React hooks
│   ├── stores/           → Global Zustand stores
│   ├── services/         → Tauri invoke wrappers
│   ├── lib/              → Utilities, constants, types
│   └── routes/           → Route definitions
├── src-tauri/
│   ├── src/
│   │   ├── commands/     → Per-module command files
│   │   ├── database/     → SQLx queries + migrations
│   │   ├── importers/    → Content importers
│   │   ├── fsrs/         → SRS engine
│   │   ├── models/       → Domain types
│   │   └── services/     → Business logic layer
│   └── migrations/       → SQLx migration files
└── content/              → Pre-bundled learning content
```
