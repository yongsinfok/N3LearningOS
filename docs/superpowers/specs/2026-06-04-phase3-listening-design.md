# JLPT N3 Learning System — Phase 3: Listening Module

**Date:** 2026-06-04
**Status:** Draft
**Version:** 1.0
**Priority:** Part 2 of Phase 3 (after Reading module)

---

## 1. Overview

Listening module for N3LearningOS. Pre-bundled MP3 audio with sentence-level transcript sync, playback speed control, and listening comprehension quiz. Follows same island architecture as reading module.

### Key Constraints

- Fully offline, pre-bundled audio (`content/audio/*.mp3`) + metadata (`content/listening.json`)
- `<audio>` element for playback (zero Rust audio deps)
- Raw MP3 bytes served via Tauri `get_audio_data` command → `Blob` → `URL.createObjectURL`
- TTS-generated (edge-tts) Japanese audio with precise timestamps
- No YouTube, no streaming, no cloud APIs

---

## 2. Architecture

### 2.1 Module Structure

```
src/features/listening/
  pages/
    ListeningListPage.tsx     → /listening
    ListeningDetailPage.tsx   → /listening/:id (player)
  components/
    AudioPlayer.tsx            → Play/pause, progress bar, speed control
    TranscriptView.tsx         → Timestamped transcript, sentence highlighting, click-to-seek
    QuizSection.tsx            → Comprehension quiz (reuses reading pattern)
  hooks/
    useListening.ts            → TanStack Query hooks
  types/
    index.ts                   → Re-exports

src-tauri/src/
  commands/listening.rs        → list_listening, get_listening_detail, get_audio_data
  importers/listening.rs       → Import pipeline
  migrations/004_listening.sql → Listening table
```

### 2.2 Audio Data Flow

```
Rust (get_audio_data):
  [content/audio/listening_001.mp3]
        → fs::read() → Vec<u8> → tauri::command returns Vec<u8>

Frontend:
  invoke("get_audio_data", { audioFile: "listening_001.mp3" })
    → Uint8Array → new Blob([data], {type: "audio/mp3"})
    → URL.createObjectURL(blob) → <audio src={url} />
    → playbackRate, currentTime, timeupdate event for sync
```

### 2.3 Routes

```
/listening       → ListeningListPage (lesson list with search)
/listening/:id   → ListeningDetailPage (player + transcript + quiz)
```

---

## 3. Data Design

### 3.1 Database Schema (Migration 004)

```sql
CREATE TABLE IF NOT EXISTS listening (
    id          TEXT PRIMARY KEY,
    title       TEXT NOT NULL,
    audio_file  TEXT NOT NULL,         -- "listening_001.mp3"
    transcript  TEXT NOT NULL,         -- JSON array of TranscriptEntry
    questions   TEXT,                  -- JSON array of Question (nullable)
    source      TEXT NOT NULL DEFAULT 'bundled',
    level       TEXT NOT NULL DEFAULT 'N3',
    is_bookmarked INTEGER DEFAULT 0,
    created_at  TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_listening_title ON listening(title);
```

### 3.2 Rust Models (add to `models/mod.rs`)

```rust
#[derive(Debug, Serialize, Deserialize, Clone, sqlx::FromRow)]
pub struct Listening {
    pub id: String,
    pub title: String,
    pub audio_file: String,
    pub transcript: String,
    pub questions: Option<String>,
    pub source: String,
    pub level: String,
    pub is_bookmarked: bool,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TranscriptEntry {
    pub start: f64,       // seconds
    pub end: f64,
    pub text: String,
}
```

Reuses existing `Question` and `Segment` from reading module. `ImportResult` already has `reading_imported` — add `listening_imported`.

### 3.3 Content Format

`content/listening.json`:

```json
{
  "version": "1.0.0",
  "items": [
    {
      "id": "listening_001",
      "title": "駅のアナウンス",
      "level": "N3",
      "audio_file": "listening_001.mp3",
      "source": "bundled",
      "transcript": [
        { "start": 0.0, "end": 2.5, "text": "次は新宿駅です。" },
        { "start": 2.5, "end": 5.0, "text": "出口は右側です。" }
      ],
      "questions": [
        {
          "id": "q1",
          "question": "次の駅はどこですか？",
          "options": ["新宿", "渋谷", "東京", "池袋"],
          "correct_index": 0,
          "explanation": "「次は新宿駅です」と言っています。"
        }
      ]
    }
  ]
}
```

Audio files live in `content/audio/listening_*.mp3`.

### 3.4 Content Topics (15 Lessons)

Scenarios that commonly appear in JLPT N3 listening section:

1. 駅のアナウンス — station announcements
2. 天気予報 — weather forecast
3. 買い物の会話 — shopping conversation
4. 道案内 — giving directions
5. レストランで — at a restaurant
6. 電話の会話 — phone conversation
7. 学校のお知らせ — school announcements
8. 職場の会話 — workplace conversation
9. ニュース — news bulletin
10. 観光案内 — tourist information
11. 病院で — at the hospital
12. 図書館の案内 — library guidance
13. バスのアナウンス — bus announcements
14. イベントの案内 — event information
15. 友達との会話 — conversation between friends

Each lesson: 6-12 sentences, 30-60 seconds audio length, 2-3 comprehension questions.

---

## 4. Rust Commands

### `list_listening`

- `invoke("list_listening", { page, pageSize, query })`
- `PaginatedResult<Listening>`
- Search on `title LIKE '%query%'`

### `get_listening_detail`

- `invoke("get_listening_detail", { id })`
- `Listening` struct with transcript/questions as raw JSON strings
- Error: `"Listening '...' not found"` if missing

### `get_audio_data`

- `invoke("get_audio_data", { audioFile: "listening_001.mp3" })`
- Returns `Vec<u8>` (raw MP3 bytes)
- Reads from `content/audio/{audioFile}` relative to app content directory
- Error: `"Audio file not found"` if MP3 missing
- Frontend handles: `Uint8Array → Blob → createObjectURL`

---

## 5. Frontend Components

### 5.1 ListeningListPage

- Same pattern as ReadingListPage
- Search bar + lesson cards with title + level badge + duration indicator
- States: loading (skeleton), empty ("暂无听力课程"), error, success

### 5.2 ListeningDetailPage (Player)

Layout:

```
┌─────────────────────────────────────┐
│ ← 返回列表     标题            🔖   │
├─────────────────────────────────────┤
│                                     │
│  [▶ ⏸] [▓▓▓▓▓░░░░░░░░░░░░░] [1x] │
│                                     │
│  0:00                   1:23 / 3:45│
│                                     │
│  ─── 字幕 ───                        │
│                                     │
│  ▸ 次は新宿駅です。                   │  ← highlighted
│    出口は右側です。                    │
│    お出口は左側にお気をつけください。   │
│                                     │
│  ─── 聴解問題 ───                    │
│                                     │
│  Q1: 次の駅はどこですか？             │
│  ○ 新宿  ○ 渋谷  ○ 東京  ○ 池袋    │
│                                     │
│  [正解/不正解 フィードバック]         │
│                                     │
└─────────────────────────────────────┘
```

### 5.3 AudioPlayer Component

Props: `audioUrl: string (blob URL)`

State machine:
- `loading` — fetching audio bytes from Tauri
- `ready` — audio loaded, can play
- `playing` — actively playing
- `paused` — paused at position
- `error` — audio load failed

Controls:
- Play/Pause toggle button
- Progress bar (range input or custom slider)
- Current time / duration display
- Speed selector: 0.75x | 1x | 1.25x | 1.5x

Implementation:
```tsx
const audioRef = useRef<HTMLAudioElement>(null);
const [currentTime, setCurrentTime] = useState(0);
const [duration, setDuration] = useState(0);
const [speed, setSpeed] = useState(1);
const [isPlaying, setIsPlaying] = useState(false);

// timeupdate → sync transcript highlight
// audioRef.current.playbackRate = speed
// Progress bar: <input type="range" min={0} max={duration} value={currentTime} />
```

### 5.4 TranscriptView Component

Props: `transcriptJson: string, currentTime: number, audioRef`

- Parses JSON transcript entries
- Maps each entry to a sentence row
- Highlights the sentence where `start <= currentTime < end`
- Click on a sentence: `audioRef.current.currentTime = entry.start; audioRef.current.play()`
- Styling: highlighted sentence in primary color, others in muted text

### 5.5 QuizSection Component

Identical pattern to reading's QuizSection. If both reading and listening use the same `Question` type, this can be a shared component at `src/shared/components/QuizSection.tsx`.

Decision: For now, copy the implementation to keep modules independent. Refactor to shared if a third module uses it.

---

## 6. Content Generation Pipeline

The Python generator script (`scripts/generate_listening.py`) is a **development tool only** — not included in the final app build.

### Workflow

```
scripts/listening_scripts.py  (raw script text per lesson — written by hand)
  ↓
generate_listening.py
  ├── Reads script text with sentence-level annotations
  ├── For each sentence:
  │     → subprocess.run(["edge-tts", "--voice=ja-JP-NanamiNeural", "--text=...", "--write-media=sentence_001.mp3"])
  │     → pydub.AudioSegment.from_mp3("sentence_001.mp3") → measure duration
  │     → Accumulate start/end timestamps
  ├── Merges all sentences → content/audio/listening_XXX.mp3
  └── Outputs content/listening.json with transcript timestamps + questions
```

### Requirements

```bash
pip install edge-tts pydub
```

edge-tts requires network access during generation (calls Microsoft TTS API). Generated files are fully offline.

### Timestamp Accuracy

- edge-tts outputs the exact duration of each sentence
- No silence trimming between sentences (natural flow)
- Each sentence's `start` = cumulative duration of all previous sentences
- Each sentence's `end` = `start` + that sentence's duration

### Fallback for edge-tts

If edge-tts fails or user can't install it, provide pre-generated MP3s directly in the repo (checked into git). The script is optional for regeneration.

---

## 7. Rust Importer

Follows same pattern as reading importer.

- `import_listening(content_dir)` → parse `content/listening.json`, returns `Vec<Listening>`
- `save_listening_to_db(items)` → INSERT OR IGNORE
- `create_listening_flashcards()` → returns `Ok(0)` (no flashcards, quiz-based assessment)
- No audio file import needed (audio stays in filesystem, read via `get_audio_data`)

Update `importers/mod.rs` to call listening import after reading. Update `ImportResult` to add `listening_imported`.

---

## 8. File Change Summary

| File | Action |
|---|---|
| `src-tauri/migrations/004_listening.sql` | Create |
| `src-tauri/src/models/mod.rs` | Add Listening, TranscriptEntry structs; update ImportResult |
| `src-tauri/src/commands/listening.rs` | Create |
| `src-tauri/src/commands/mod.rs` | Add module |
| `src-tauri/src/importers/listening.rs` | Create |
| `src-tauri/src/importers/mod.rs` | Wire listening into pipeline |
| `src-tauri/src/lib.rs` | Register commands, update import log |
| `content/listening.json` | Create (15 lessons) |
| `content/audio/listening_*.mp3` | Add (15 MP3 files) |
| `content/manifest.json` | Add listening entry |
| `src/features/listening/pages/ListeningListPage.tsx` | Create |
| `src/features/listening/pages/ListeningDetailPage.tsx` | Create |
| `src/features/listening/components/AudioPlayer.tsx` | Create |
| `src/features/listening/components/TranscriptView.tsx` | Create |
| `src/features/listening/components/QuizSection.tsx` | Create |
| `src/features/listening/hooks/useListening.ts` | Create |
| `src/features/listening/types/index.ts` | Create |
| `src/routes/index.tsx` | Add listening routes |
| `src/services/api.ts` | Add listening API functions |
| `src-tauri/src/commands/dashboard.rs` | Update module_progress to include listening |
| `scripts/generate_listening.py` | Create (dev tool, optional) |

---

## 9. Error Handling

| Scenario | Frontend | Backend |
|---|---|---|
| Listening ID not found | "课程不存在" + redirect | `Err("Listening '...' not found")` |
| Audio file not found | "音频加载失败" + retry | `Err("Audio file not found")` |
| Audio decode error | "音频格式不支持" | N/A (frontend `<audio>` handles) |
| Empty listening list | "暂无听力课程" | Empty PaginatedResult |
| Search no results | "没有找到相关课程" | Empty PaginatedResult |
| DB connection fail | Error toast + retry | `Err(database error)` |

---

## 10. Testing

### Backend

- `test_import_listening` — import listening.json, verify rows in DB
- `test_get_audio_data` — read known MP3, verify Vec<u8> length > 0
- `test_get_audio_data_missing` — request unknown file, verify error

### Frontend

- `AudioPlayer` — play/pause toggle, progress bar interaction, speed change
- `TranscriptView` — sentence highlighting matches currentTime, click seeks audio
