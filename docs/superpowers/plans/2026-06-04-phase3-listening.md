# Phase 3: Listening Module Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use subagent-driven-development or executing-plans to implement.

**Goal:** Add JLPT N3 listening module with audio player, transcript sync, speed control, and comprehension quiz.

**Architecture:** `<audio>` element + Blob URL (bytes via Tauri command). Same island pattern as reading module. No Rust audio deps.

**Tech Stack:** Tauri v2, Rust/SQLx, React 19, Tailwind v4, shadcn/ui, TanStack Query

**Spec:** `docs/superpowers/specs/2026-06-04-phase3-listening-design.md`

---

### Task 1: Migration 004 + Rust models

**Files:**
- Create: `src-tauri/migrations/004_listening.sql`
- Modify: `src-tauri/src/models/mod.rs`

**Step 1: Create 004_listening.sql**

```sql
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
```

**Step 2: Add Listening + TranscriptEntry structs to models/mod.rs**

After `Question` struct, add:

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
    pub start: f64,
    pub end: f64,
    pub text: String,
}
```

Update `ImportResult` — add `listening_imported: usize` after `reading_imported`. Then update both construction sites in `importers/mod.rs` (early return + main return) to include `listening_imported: 0` / `listening_imported: listening_count`.

**Step 3: Build**

```bash
cd src-tauri && cargo build 2>&1 | tail -5
```

---

### Task 2: Listening importer

**Files:**
- Create: `src-tauri/src/importers/listening.rs`
- Modify: `src-tauri/src/importers/mod.rs`

**Step 1: Create importers/listening.rs**

```rust
use crate::models::Listening;
use std::fs;
use std::path::Path;

pub fn import_listening(content_dir: &Path) -> Result<Vec<Listening>, String> {
    let path = content_dir.join("listening.json");
    if !path.exists() { return Err("listening.json not found".into()); }
    let content = fs::read_to_string(&path).map_err(|e| format!("Cannot read: {}", e))?;
    let items: Vec<ListeningInput> = serde_json::from_str(&content).map_err(|e| format!("Cannot parse: {}", e))?;
    let now = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S").to_string();
    let items: Vec<Listening> = items.into_iter().map(|r| Listening {
        id: r.id.unwrap_or_else(|| uuid::Uuid::new_v4().to_string()),
        title: r.title,
        audio_file: r.audio_file,
        transcript: serde_json::to_string(&r.transcript).unwrap_or_default(),
        questions: r.questions.map(|q| serde_json::to_string(&q).unwrap_or_default()),
        source: r.source.unwrap_or_else(|| "bundled".into()),
        level: r.level.unwrap_or_else(|| "N3".into()),
        is_bookmarked: false,
        created_at: now.clone(),
    }).collect();
    Ok(items)
}

pub fn save_listening_to_db(items: &[Listening]) -> Result<usize, sqlx::Error> {
    let pool = crate::database::get_pool().map_err(|e| sqlx::Error::Protocol(e.to_string()))?;
    futures::executor::block_on(async {
        let mut count = 0;
        for r in items {
            sqlx::query("INSERT OR IGNORE INTO listening (id, title, audio_file, transcript, questions, source, level, is_bookmarked, created_at) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9)")
                .bind(&r.id).bind(&r.title).bind(&r.audio_file).bind(&r.transcript)
                .bind(&r.questions).bind(&r.source).bind(&r.level).bind(&r.is_bookmarked).bind(&r.created_at)
                .execute(pool).await?;
            count += 1;
        }
        Ok(count)
    })
}

pub fn create_listening_flashcards() -> Result<usize, sqlx::Error> { Ok(0) }

#[derive(Debug, serde::Deserialize)]
struct ListeningInput {
    id: Option<String>,
    title: String,
    audio_file: String,
    transcript: Vec<TranscriptEntry>,
    questions: Option<Vec<crate::models::Question>>,
    source: Option<String>,
    level: Option<String>,
}
```

**Note:** Need `use crate::models::TranscriptEntry;` or reference it via `crate::models::TranscriptEntry` in ListeningInput. I prefer the inline path.

Actually simpler — just use the full path. Import `use crate::models::TranscriptEntry;` at top.

Wait, `crate::models::TranscriptEntry` is fine inline since it's only used in the ListeningInput struct.

**Step 2: Wire into importers/mod.rs**

Add `pub mod listening;` after `pub mod reading;`. After reading block (line 47), add:

```rust
    let listening = listening::import_listening(content_dir)?;
    let listening_count = listening::save_listening_to_db(&listening)
        .map_err(|e| format!("Listening DB insert failed: {}", e))?;
```

Update ImportResult: change `listening_imported: 0` in early return, and add `listening_imported: listening_count` in main return.

**Step 3: Build**

```bash
cd src-tauri && cargo build 2>&1 | tail -5
```

---

### Task 3: Rust commands for listening

**Files:**
- Create: `src-tauri/src/commands/listening.rs`
- Modify: `src-tauri/src/commands/mod.rs`
- Modify: `src-tauri/src/lib.rs`

**Step 1: Create commands/listening.rs**

```rust
use crate::database::get_pool;
use crate::models::{Listening, PaginatedResult};

#[tauri::command]
pub async fn list_listening(
    page: Option<u32>,
    search: Option<String>,
    page_size: Option<u32>,
) -> Result<PaginatedResult<Listening>, String> {
    let pool = get_pool().map_err(|e| e.to_string())?;
    let page = page.unwrap_or(1);
    let page_size = page_size.unwrap_or(20).min(100);
    let offset = ((page.saturating_sub(1)) * page_size) as i64;
    if let Some(ref term) = search {
        let pattern = format!("%{}%", term);
        let count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM listening WHERE title LIKE ?1")
            .bind(&pattern).fetch_one(pool).await.map_err(|e| e.to_string())?;
        let items = sqlx::query_as::<_, Listening>("SELECT * FROM listening WHERE title LIKE ?1 ORDER BY title LIMIT ?2 OFFSET ?3")
            .bind(&pattern).bind(page_size as i64).bind(offset).fetch_all(pool).await.map_err(|e| e.to_string())?;
        Ok(PaginatedResult { items, total: count, page, page_size })
    } else {
        let count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM listening")
            .fetch_one(pool).await.map_err(|e| e.to_string())?;
        let items = sqlx::query_as::<_, Listening>("SELECT * FROM listening ORDER BY title LIMIT ?1 OFFSET ?2")
            .bind(page_size as i64).bind(offset).fetch_all(pool).await.map_err(|e| e.to_string())?;
        Ok(PaginatedResult { items, total: count, page, page_size })
    }
}

#[tauri::command]
pub async fn get_listening_detail(id: String) -> Result<Listening, String> {
    let pool = get_pool().map_err(|e| e.to_string())?;
    sqlx::query_as::<_, Listening>("SELECT * FROM listening WHERE id = ?1")
        .bind(&id).fetch_optional(pool).await.map_err(|e| e.to_string())?
        .ok_or_else(|| format!("Listening '{}' not found", id))
}

#[tauri::command]
pub async fn get_audio_data(audio_file: String) -> Result<Vec<u8>, String> {
    let content_dir = vec![]; // TODO: need content_dir path
    // Actually, we need the content dir at runtime. Best approach: read from known path relative to exe.
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    let exe_dir = exe.parent().unwrap_or(&std::path::PathBuf::from(".")).to_path_buf();
    let audio_path = exe_dir.join("content").join("audio").join(&audio_file);
    if !audio_path.exists() {
        return Err(format!("Audio file '{}' not found", audio_file));
    }
    std::fs::read(&audio_path).map_err(|e| format!("Cannot read audio: {}", e))
}
```

Wait, this duplicates `get_content_dir()` from lib.rs. Better to extract it or reference it. Since `get_content_dir` is private in lib.rs, I should either make it pub or duplicate. Simplest: inline the path logic.

Actually there's a cleaner approach. The content dir is relative to the executable. The `get_content_dir()` function in lib.rs does `exe_dir.join("content")`. For the audio command, we do `content_dir.join("audio").join(audio_file)`. Let me make `get_content_dir()` public.

Modify `lib.rs` — change `fn get_content_dir()` to `pub fn get_content_dir()`.

Then in `commands/listening.rs`:

```rust
#[tauri::command]
pub async fn get_audio_data(audio_file: String) -> Result<Vec<u8>, String> {
    let content_dir = crate::get_content_dir();
    let audio_path = content_dir.join("audio").join(&audio_file);
    if !audio_path.exists() {
        return Err(format!("Audio file '{}' not found", audio_file));
    }
    std::fs::read(&audio_path).map_err(|e| format!("Cannot read audio: {}", e))
}
```

**Step 2: Register mod** in commands/mod.rs — add `pub mod listening;`.

**Step 3: Register in lib.rs** — make get_content_dir pub, add 3 commands to invoke_handler, update import log.

**Step 4: Build**

```bash
cd src-tauri && cargo build 2>&1 | tail -5
```

---

### Task 4: Content — 15 listening lessons + MP3 + generation script

**Files:**
- Create: `content/listening.json`
- Create: `content/audio/` (15 MP3 files)
- Create: `scripts/generate_listening.py`
- Modify: `content/manifest.json`

**Step 1: Write listening scripts**

15 files or one big Python file with all scripts embedded. Recommend: one `scripts/listening_data.py` with all 15 lesson scripts + questions. The generator reads that file.

Or simplest: the generator script contains all 15 lesson texts as inline data, calls edge-tts per sentence, merges outputs, writes JSON.

I'll provide a generation script that:
1. Has all 15 lesson scripts hardcoded
2. For each sentence: calls `edge-tts --voice=ja-JP-NanamiNeural --text="..." --write-media=/tmp/sentence_N.wav`
3. Uses pydub to merge and get durations
4. Outputs `content/audio/listening_XXX.mp3` and `content/listening.json`

If edge-tts isn't available, the script can also work with pre-placed MP3 files.

**Step 2: Update manifest.json** — add `{ "path": "listening.json", "type": "listening", "checksum": null }`, bump version to 1.3.0.

---

### Task 5: Frontend types + API + hooks

**Files:**
- Modify: `src/services/api.ts`
- Create: `src/features/listening/types/index.ts`
- Create: `src/features/listening/hooks/useListening.ts`

**Step 1: Add to api.ts**

After Reading section, add:

```ts
// === Listening ===
export interface TranscriptEntry {
  start: number;
  end: number;
  text: string;
}

export interface Listening {
  id: string;
  title: string;
  audio_file: string;
  transcript: string;    // JSON
  questions: string | null;
  source: string;
  level: string;
  is_bookmarked: boolean;
  created_at: string;
}

export function listListening(page?: number, search?: string, pageSize?: number): Promise<PaginatedResult<Listening>> {
  return invoke("list_listening", { page, search, page_size: pageSize });
}

export function getListeningDetail(id: string): Promise<Listening> {
  return invoke("get_listening_detail", { id });
}

export function getAudioData(audioFile: string): Promise<number[]> {
  return invoke("get_audio_data", { audio_file: audioFile });
}
```

Note: Tauri returns `Vec<u8>` as `number[]` in JS. We convert to `Uint8Array`.

**Step 2: Create types/index.ts**

```ts
export type { Listening, TranscriptEntry, Question, PaginatedResult } from "@/services/api";
```

**Step 3: Create hooks/useListening.ts**

```ts
import { useQuery } from "@tanstack/react-query";
import { listListening, getListeningDetail } from "@/services/api";

export function useListeningList(page = 1, search?: string) {
  return useQuery({
    queryKey: ["listening", "list", page, search],
    queryFn: () => listListening(page, search, 20),
  });
}

export function useListeningDetail(id: string) {
  return useQuery({
    queryKey: ["listening", id],
    queryFn: () => getListeningDetail(id),
    enabled: !!id,
  });
}
```

---

### Task 6: AudioPlayer component

**Files:**
- Create: `src/features/listening/components/AudioPlayer.tsx`

**Component:**

```tsx
import { useState, useRef, useEffect, useCallback } from "react";
import { Button } from "@/components/ui/button";
import { getAudioData } from "@/services/api";

interface Props {
  audioFile: string;
  onTimeUpdate?: (time: number) => void;
  onDuration?: (duration: number) => void;
}

export default function AudioPlayer({ audioFile, onTimeUpdate, onDuration }: Props) {
  const audioRef = useRef<HTMLAudioElement>(null);
  const [audioUrl, setAudioUrl] = useState<string | null>(null);
  const [isPlaying, setIsPlaying] = useState(false);
  const [currentTime, setCurrentTime] = useState(0);
  const [duration, setDuration] = useState(0);
  const [speed, setSpeed] = useState(1);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState(false);

  useEffect(() => {
    setLoading(true);
    setError(false);
    getAudioData(audioFile)
      .then((bytes) => {
        const blob = new Blob([new Uint8Array(bytes)], { type: "audio/mp3" });
        const url = URL.createObjectURL(blob);
        setAudioUrl(url);
        setLoading(false);
      })
      .catch(() => { setError(true); setLoading(false); });
    return () => { if (audioUrl) URL.revokeObjectURL(audioUrl); };
  }, [audioFile]);

  // Play/Pause
  const togglePlay = useCallback(() => {
    if (!audioRef.current) return;
    if (audioRef.current.paused) {
      audioRef.current.play().then(() => setIsPlaying(true)).catch(() => {});
    } else {
      audioRef.current.pause();
      setIsPlaying(false);
    }
  }, []);

  // Speed
  const cycleSpeed = useCallback(() => {
    const speeds = [0.75, 1, 1.25, 1.5];
    const idx = speeds.indexOf(speed);
    const next = speeds[(idx + 1) % speeds.length];
    setSpeed(next);
    if (audioRef.current) audioRef.current.playbackRate = next;
  }, [speed]);

  // Seek
  const handleSeek = useCallback((e: React.ChangeEvent<HTMLInputElement>) => {
    const time = Number(e.target.value);
    setCurrentTime(time);
    if (audioRef.current) audioRef.current.currentTime = time;
  }, []);

  // Format time mm:ss
  const fmt = (s: number) => `${Math.floor(s / 60)}:${String(Math.floor(s % 60)).padStart(2, "0")}`;

  if (loading) return <div className="flex items-center justify-center py-8"><p className="text-muted-foreground">加载音频中...</p></div>;
  if (error) return <div className="flex items-center justify-center py-8"><p className="text-destructive">音频加载失败</p></div>;

  return (
    <div className="space-y-3 rounded-lg border p-4">
      <audio
        ref={audioRef}
        src={audioUrl!}
        preload="auto"
        onTimeUpdate={() => {
          const t = audioRef.current?.currentTime ?? 0;
          setCurrentTime(t);
          onTimeUpdate?.(t);
        }}
        onLoadedMetadata={() => {
          const d = audioRef.current?.duration ?? 0;
          setDuration(d);
          onDuration?.(d);
        }}
        onEnded={() => setIsPlaying(false)}
      />

      <div className="flex items-center gap-3">
        <Button variant="outline" size="icon" onClick={togglePlay}>
          {isPlaying ? "⏸" : "▶"}
        </Button>

        <input
          type="range"
          min={0}
          max={duration || 0}
          value={currentTime}
          onChange={handleSeek}
          className="flex-1"
        />

        <span className="min-w-[90px] text-right text-sm tabular-nums text-muted-foreground">
          {fmt(currentTime)} / {fmt(duration)}
        </span>

        <Button variant="outline" size="sm" onClick={cycleSpeed}>
          {speed}x
        </Button>
      </div>
    </div>
  );
}
```

---

### Task 7: TranscriptView component

**Files:**
- Create: `src/features/listening/components/TranscriptView.tsx`

```tsx
import { useMemo, useRef, useEffect } from "react";
import { TranscriptEntry } from "@/services/api";

interface Props {
  transcriptJson: string;
  currentTime: number;
  onSeek: (time: number) => void;
}

export default function TranscriptView({ transcriptJson, currentTime, onSeek }: Props) {
  const entries: TranscriptEntry[] = useMemo(() => {
    try { return JSON.parse(transcriptJson); } catch { return []; }
  }, [transcriptJson]);

  const activeIndex = entries.findIndex(
    (e) => currentTime >= e.start && currentTime < e.end,
  );
  const activeRef = useRef<HTMLParagraphElement>(null);

  useEffect(() => {
    if (activeRef.current) {
      activeRef.current.scrollIntoView({ behavior: "smooth", block: "center" });
    }
  }, [activeIndex]);

  if (entries.length === 0) return null;

  return (
    <div className="space-y-2">
      <h3 className="font-semibold">字幕</h3>
      <div className="max-h-64 space-y-1 overflow-y-auto rounded-md border p-3">
        {entries.map((entry, i) => (
          <p
            key={i}
            ref={i === activeIndex ? activeRef : undefined}
            className={`cursor-pointer rounded px-2 py-1 text-sm transition-colors ${
              i === activeIndex
                ? "bg-primary text-primary-foreground font-medium"
                : "text-muted-foreground hover:bg-muted"
            }`}
            onClick={() => onSeek(entry.start)}
          >
            {entry.text}
          </p>
        ))}
      </div>
    </div>
  );
}
```

---

### Task 8: PlaylistPage + DetailPage

**Files:**
- Create: `src/features/listening/pages/ListeningListPage.tsx`
- Create: `src/features/listening/pages/ListeningDetailPage.tsx`
- Create: `src/features/listening/components/QuizSection.tsx`

**Step 1: ListeningListPage**

Same pattern as ReadingListPage. Replace `useReadingList` → `useListeningList`, "阅读" → "听力", `/reading/` → `/listening/`.

**Step 2: ListeningDetailPage**

```tsx
import { useState, useCallback } from "react";
import { useParams, useNavigate } from "react-router-dom";
import { Button } from "@/components/ui/button";
import { Skeleton } from "@/components/ui/skeleton";
import { useListeningDetail } from "../hooks/useListening";
import AudioPlayer from "../components/AudioPlayer";
import TranscriptView from "../components/TranscriptView";
import QuizSection from "../components/QuizSection";

export default function ListeningDetailPage() {
  const { id } = useParams<{ id: string }>();
  const { data, isLoading, error } = useListeningDetail(id!);
  const navigate = useNavigate();
  const [currentTime, setCurrentTime] = useState(0);
  const audioRef = useRef<HTMLAudioElement>(null);

  const handleSeek = useCallback((time: number) => {
    if (audioRef.current) {
      audioRef.current.currentTime = time;
      audioRef.current.play();
    }
    setCurrentTime(time);
  }, []);

  if (isLoading) {
    return (
      <div className="space-y-4">
        <Skeleton className="h-8 w-32" />
        <Skeleton className="h-24 w-full" />
        <Skeleton className="h-48 w-full" />
      </div>
    );
  }

  if (error || !data) {
    return (
      <div className="flex flex-col items-center gap-3 py-12">
        <p className="text-destructive">课程不存在</p>
        <Button variant="outline" onClick={() => navigate("/listening")}>返回列表</Button>
      </div>
    );
  }

  return (
    <div className="mx-auto max-w-2xl space-y-6">
      <Button variant="ghost" onClick={() => navigate("/listening")}>← 返回列表</Button>
      <h1 className="text-2xl font-bold">{data.title}</h1>

      <AudioPlayer
        audioFile={data.audio_file}
        onTimeUpdate={setCurrentTime}
      />

      <TranscriptView
        transcriptJson={data.transcript}
        currentTime={currentTime}
        onSeek={handleSeek}
      />

      <QuizSection questionsJson={data.questions} />
    </div>
  );
}
```

Wait, need `useRef` import. Add `import { useRef } from "react";`.

Also, ReadingDetailPage uses `useState` for popover — ListeningDetailPage needs `useState` for `currentTime` and `useRef` for `audioRef`. But `AudioPlayer` internally uses its own `audioRef` — we need to communicate seek externally. Option A: lift audio ref up, Option B: expose `onSeek` callback from AudioPlayer.

Simplest: AudioPlayer accepts a `seekTo?: number` prop. If `seekTo` changes and is non-null, AudioPlayer sets `currentTime`. But that's fragile.

Better: Keep AudioPlayer self-contained. For seek-from-transcript, have TranscriptView emit a seek event, and AudioPlayer exposes a `ref` or imperative handle.

Actually simplest: lift the `<audio>` element ref into the page component and pass it to both AudioPlayer and TranscriptView. But that breaks AudioPlayer encapsulation.

Let me just keep it simple — the AudioPlayer has the `<audio>` ref internally. For seeking from transcript, the `onSeek` callback in DetailPage tries `document.querySelector('audio')` to set currentTime. It's not pretty but works.

Even simpler: put the `<audio>` element in DetailPage, not in AudioPlayer. AudioPlayer becomes a set of controls that receives an audioRef. But that's a rearch.

OR — just pass a seek signal. AudioPlayer accepts `externalSeek: number | null`. When it changes, AudioPlayer applies it.

Let me go with the pragmatic approach: the DetailPage has a callback `handleSeek` that sets audio `currentTime` using `document.querySelector('audio')`. The AudioPlayer component contains the `<audio>` element with an id.

Actually, let me use a different pattern — `useImperativeHandle` or simply put the `<audio>` HTML element directly in the page and have AudioPlayer be just controls. But then AudioPlayer can't manage its own state easily.

Simplest working approach: put `<audio>` in DetailPage, pass ref to AudioPlayer. 

```tsx
// DetailPage
const audioRef = useRef<HTMLAudioElement>(null);

<audio ref={audioRef} src={audioUrl} preload="auto" ... />
<AudioPlayer audioRef={audioRef} audioFile={data.audio_file} onTimeUpdate={setCurrentTime} />
<TranscriptView ... onSeek={(t) => { if (audioRef.current) { audioRef.current.currentTime = t; audioRef.current.play(); }}} />
```

Wait, but AudioPlayer loads the audio bytes and creates the blob URL. That's the part that needs to feed into `<audio>`. So if we move `<audio>` to DetailPage, AudioPlayer needs to return the blob URL.

Let me refactor: AudioPlayer handles loading audio bytes and creating blob URL, then passes the URL up via a callback. DetailPage puts `<audio>` in its own JSX.

Actually this is getting too complex for the plan. Let me keep it simple: AudioPlayer manages `<audio>` internally, exposes `onSeek` via parent, and the parent communicates seek by setting a value on the DOM element.

Best pragmatic approach: In the DetailPage, when `handleSeek` is called, find the audio element via querySelector and set its currentTime. It's a single-line hack but avoids all the prop-drilling complexity.

```tsx
const handleSeek = useCallback((time: number) => {
  const audio = document.querySelector("audio");
  if (audio) { audio.currentTime = time; audio.play(); }
}, []);
```

It works perfectly for this use case. Let's go with it.

**Step 3: QuizSection**

Copy from reading module (`src/features/reading/components/QuizSection.tsx`). Both use the same `Question` type. No changes needed except file location.

Actually wait — we should create `src/features/listening/components/QuizSection.tsx` instead of importing the reading one. Keeps modules decoupled.

---

### Task 9: Routes + Dashboard

**Files:**
- Modify: `src/routes/index.tsx`
- Modify: `src-tauri/src/commands/dashboard.rs`

**Step 1: Routes**

Add imports:
```tsx
import ListeningListPage from "@/features/listening/pages/ListeningListPage";
import ListeningDetailPage from "@/features/listening/pages/ListeningDetailPage";
```

Add routes after reading routes:
```tsx
{ path: "listening", element: <ListeningListPage /> },
{ path: "listening/:id", element: <ListeningDetailPage /> },
```

**Step 2: Dashboard**

In `dashboard.rs`, after reading section, add:
```rust
    let listening_total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM listening")
        .fetch_one(pool).await.map_err(|e| e.to_string())?;
    let listening_done: i64 = 0;
```

Add to `module_progress`:
```rust
    ModuleProgress { module: "listening".into(), completed: listening_done, total: listening_total },
```

---

### Task 10: Build + verify

**Step 1: Rust build**
```bash
cd src-tauri && cargo build 2>&1 | tail -10
```

**Step 2: TypeScript check**
```bash
cd /home/joshua/vibe/N3LearningOS/.claude/worktrees/phase3+listening && npx tsc --noEmit 2>&1
```

**Step 3: Run tests**
```bash
cd src-tauri && cargo test 2>&1 | tail -10
```

All must pass with 0 errors.
