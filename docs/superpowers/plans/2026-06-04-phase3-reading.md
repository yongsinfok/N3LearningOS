# Phase 3: Reading Module Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add JLPT N3 reading module with article reader, furigana toggle, word lookup popup, and comprehension quiz.

**Architecture:** Follows island architecture established by vocab/grammar/kanji modules. Pre-bundled content with embedded furigana (no wasm kuromoji). No flashcards for reading (quiz-based assessment). Rust commands via Tauri invoke, React/TanStack Query frontend.

**Tech Stack:** Tauri v2, Rust/SQLx, React 19, TypeScript, Tailwind v4, shadcn/ui, TanStack Query

**Spec:** `docs/superpowers/specs/2026-06-04-phase3-reading-design.md`

---

### Task 1: Database migration 003 + Rust models

**Files:**
- Create: `src-tauri/migrations/003_reading.sql`
- Modify: `src-tauri/src/models/mod.rs`
- Modify: `src-tauri/src/importers/mod.rs` (ImportResult struct)

---

- [ ] **Step 1: Create migration 003_reading.sql**

```sql
-- src-tauri/migrations/003_reading.sql
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
```

---

- [ ] **Step 2: Add Reading/Segment/Question structs + update ImportResult in models/mod.rs**

After the `Kanji` struct (line ~65), add:

```rust
#[derive(Debug, Serialize, Deserialize, Clone, sqlx::FromRow)]
pub struct Reading {
    pub id: String,
    pub title: String,
    pub segments: String,        // JSON text
    pub questions: Option<String>,
    pub source: String,
    pub level: String,
    pub is_bookmarked: bool,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Segment {
    pub t: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub f: Option<String>,
    pub k: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Question {
    pub id: String,
    pub question: String,
    pub options: Vec<String>,
    pub correct_index: u8,
    pub explanation: String,
}
```

Update `ImportResult` to add `reading_imported`:

```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct ImportResult {
    pub vocabulary_imported: usize,
    pub grammar_imported: usize,
    pub kanji_imported: usize,
    pub reading_imported: usize,
    pub flashcards_created: usize,
    pub success: bool,
}
```

---

- [ ] **Step 3: Build check**

Run: `cd src-tauri && cargo build 2>&1 | head -20`
Expected: Compilation errors only if missing imports

---

### Task 2: Reading importer module

**Files:**
- Create: `src-tauri/src/importers/reading.rs`
- Modify: `src-tauri/src/importers/mod.rs`

---

- [ ] **Step 1: Create reading importer**

```rust
use crate::models::Reading;
use std::fs;
use std::path::Path;

pub fn import_reading(content_dir: &Path) -> Result<Vec<Reading>, String> {
    let path = content_dir.join("reading.json");
    if !path.exists() { return Err("reading.json not found".into()); }

    let content = fs::read_to_string(&path).map_err(|e| format!("Cannot read: {}", e))?;
    let items: Vec<ReadingInput> = serde_json::from_str(&content).map_err(|e| format!("Cannot parse: {}", e))?;

    let now = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S").to_string();
    let readings: Vec<Reading> = items.into_iter().map(|r| Reading {
        id: r.id.unwrap_or_else(|| uuid::Uuid::new_v4().to_string()),
        title: r.title,
        segments: serde_json::to_string(&r.segments).unwrap_or_default(),
        questions: r.questions.map(|q| serde_json::to_string(&q).unwrap_or_default()),
        source: r.source.unwrap_or_else(|| "bundled".into()),
        level: r.level.unwrap_or_else(|| "N3".into()),
        is_bookmarked: false,
        created_at: now.clone(),
    }).collect();

    Ok(readings)
}

pub fn save_reading_to_db(items: &[Reading]) -> Result<usize, sqlx::Error> {
    let pool = crate::database::get_pool().map_err(|e| sqlx::Error::Protocol(e.to_string()))?;
    futures::executor::block_on(async {
        let mut count = 0;
        for r in items {
            sqlx::query(
                "INSERT OR IGNORE INTO reading (id, title, segments, questions, source, level, is_bookmarked, created_at) VALUES (?1,?2,?3,?4,?5,?6,?7,?8)",
            )
            .bind(&r.id).bind(&r.title).bind(&r.segments).bind(&r.questions)
            .bind(&r.source).bind(&r.level).bind(&r.is_bookmarked).bind(&r.created_at)
            .execute(pool).await?;
            count += 1;
        }
        Ok(count)
    })
}

#[derive(Debug, serde::Deserialize)]
struct ReadingInput {
    id: Option<String>,
    title: String,
    segments: Vec<Vec<crate::models::Segment>>,
    questions: Option<Vec<crate::models::Question>>,
    source: Option<String>,
    level: Option<String>,
}

// No flashcards for reading — assessment done via comprehension quiz
pub fn create_reading_flashcards() -> Result<usize, sqlx::Error> {
    Ok(0)
}
```

---

- [ ] **Step 2: Wire reading into importers/mod.rs**

Add `pub mod reading;` at top. In `run_initial_import()`, add after kanji import:

```rust
    let reading = reading::import_reading(content_dir)?;
    let reading_count = reading::save_reading_to_db(&reading)
        .map_err(|e| format!("DB insert failed: {}", e))?;
```

Update `total_fc` to stay the same (no reading flashcards). Update `ImportResult` return:

```rust
    Ok(ImportResult {
        vocabulary_imported: vocab_count,
        grammar_imported: grammar_count,
        kanji_imported: kanji_count,
        reading_imported: reading_count,
        flashcards_created: total_fc,
        success: true,
    })
```

Update the lib.rs setup log to show reading count:

```rust
    println!(
        "Import: {} vocabulary, {} grammar, {} kanji, {} reading, {} flashcards",
        result.vocabulary_imported,
        result.grammar_imported,
        result.grammar_imported,  // fix: should be grammar_imported but that's a bug I'm just preserving
    );
```

Actually, update the print in lib.rs to actually show all counts:

```rust
    println!(
        "Import: {} vocabulary, {} grammar, {} kanji, {} reading, {} flashcards",
        result.vocabulary_imported,
        result.grammar_imported,
        result.kanji_imported,
        result.reading_imported,
        result.flashcards_created,
    );
```

---

### Task 3: Rust commands for reading

**Files:**
- Create: `src-tauri/src/commands/reading.rs`
- Modify: `src-tauri/src/commands/mod.rs`
- Modify: `src-tauri/src/lib.rs`

---

- [ ] **Step 1: Create commands/reading.rs**

```rust
use crate::database::get_pool;
use crate::models::{PaginatedResult, Reading, Vocabulary};

#[tauri::command]
pub async fn list_reading(
    page: Option<u32>,
    search: Option<String>,
    page_size: Option<u32>,
) -> Result<PaginatedResult<Reading>, String> {
    let pool = get_pool().map_err(|e| e.to_string())?;
    let page = page.unwrap_or(1);
    let page_size = page_size.unwrap_or(20).min(100);
    let offset = ((page.saturating_sub(1)) * page_size) as i64;

    if let Some(ref term) = search {
        let pattern = format!("%{}%", term);
        let count = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM reading WHERE title LIKE ?1",
        )
        .bind(&pattern)
        .fetch_one(pool).await.map_err(|e| e.to_string())?;

        let items = sqlx::query_as::<_, Reading>(
            "SELECT * FROM reading WHERE title LIKE ?1 ORDER BY title LIMIT ?2 OFFSET ?3",
        )
        .bind(&pattern).bind(page_size as i64).bind(offset)
        .fetch_all(pool).await.map_err(|e| e.to_string())?;

        return Ok(PaginatedResult { items, total: count, page, page_size });
    } else {
        let count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM reading")
            .fetch_one(pool).await.map_err(|e| e.to_string())?;

        let items = sqlx::query_as::<_, Reading>(
            "SELECT * FROM reading ORDER BY title LIMIT ?1 OFFSET ?2",
        )
        .bind(page_size as i64).bind(offset)
        .fetch_all(pool).await.map_err(|e| e.to_string())?;

        return Ok(PaginatedResult { items, total: count, page, page_size });
    }
}

#[tauri::command]
pub async fn get_reading_detail(id: String) -> Result<Reading, String> {
    let pool = get_pool().map_err(|e| e.to_string())?;
    sqlx::query_as::<_, Reading>("SELECT * FROM reading WHERE id = ?1")
        .bind(&id)
        .fetch_optional(pool)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("Reading '{}' not found", id))
}

#[tauri::command]
pub async fn lookup_word(word: String) -> Result<Option<Vocabulary>, String> {
    let pool = get_pool().map_err(|e| e.to_string())?;
    let result = sqlx::query_as::<_, Vocabulary>(
        "SELECT * FROM vocabulary WHERE word = ?1 LIMIT 1",
    )
    .bind(&word)
    .fetch_optional(pool)
    .await
    .map_err(|e| e.to_string())?;
    Ok(result)
}
```

---

- [ ] **Step 2: Register in commands/mod.rs**

```rust
pub mod reading;
```

---

- [ ] **Step 3: Register in lib.rs invoke_handler**

Add `commands::reading::list_reading,`
Add `commands::reading::get_reading_detail,`
Add `commands::reading::lookup_word,`

Update the setup hook's println to show reading count (as shown in Task 2 Step 2).

---

- [ ] **Step 4: Build check**

Run: `cd src-tauri && cargo build 2>&1 | head -30`
Expected: Compilation succeeds

---

### Task 4: Content package — 15 N3 reading articles

**Files:**
- Create: `content/reading.json`
- Modify: `content/manifest.json`

---

- [ ] **Step 1: Create content/reading.json with 15 N3 articles**

Each article has:
- Unique id like `reading_001` through `reading_015`
- Title in Japanese
- 5-10 paragraphs, each paragraph is array of segments
- Segments: `{t, f? (furigana), k (is kanji)}`
- 2-3 comprehension questions per article
- Topics: daily life, culture, technology, environment, health, travel, food, education, work, society

Articles to write:
1. 日本の電車文化 — train culture
2. コンビニの便利さ — convenience stores
3. 四季のある日本 — four seasons
4. 健康的な食生活 — healthy eating
5. リサイクルの大切さ — recycling importance
6. 図書館の使い方 — how to use libraries
7. 日本の祭り — Japanese festivals
8. インターネットの影響 — internet impact
9. 旅行の楽しみ — joy of travel
10. ボランティア活動 — volunteer activities
11. 環境問題 — environmental issues
12. 日本の学校生活 — Japanese school life
13. アルバイト探し — finding part-time work
14. 交通ルール — traffic rules
15. 新聞とテレビ — newspapers and TV

Each article: ~150-250 words, N3-level grammar/vocabulary.

---

- [ ] **Step 2: Update content/manifest.json**

```json
{ "path": "reading.json", "type": "reading", "checksum": null }
```

Add as 4th entry in `files` array.

---

### Task 5: Frontend types + API + hooks

**Files:**
- Create: `src/features/reading/types/index.ts`
- Create: `src/features/reading/hooks/useReading.ts`
- Modify: `src/services/api.ts`

---

- [ ] **Step 1: Create types/index.ts**

```ts
export type { Reading, Segment, Question, PaginatedResult } from "@/services/api";
```

---

- [ ] **Step 2: Add Reading types to api.ts**

Add after Kanji types (~line 130):

```ts
// === Reading ===
export interface Segment {
  t: string;
  f?: string;
  k: boolean;
}

export interface Question {
  id: string;
  question: string;
  options: string[];
  correct_index: number;
  explanation: string;
}

export interface Reading {
  id: string;
  title: string;
  segments: string;    // JSON string — parse at usage
  questions: string | null;
  source: string;
  level: string;
  is_bookmarked: boolean;
  created_at: string;
}

export function listReading(page?: number, search?: string, pageSize?: number): Promise<PaginatedResult<Reading>> {
  return invoke("list_reading", { page, search, page_size: pageSize });
}

export function getReadingDetail(id: string): Promise<Reading> {
  return invoke("get_reading_detail", { id });
}

export function lookupWord(word: string): Promise<Vocabulary | null> {
  return invoke("lookup_word", { word });
}
```

---

- [ ] **Step 3: Create hooks/useReading.ts**

```ts
import { useQuery } from "@tanstack/react-query";
import { listReading, getReadingDetail } from "@/services/api";

export function useReadingList(page = 1, search?: string) {
  return useQuery({
    queryKey: ["reading", "list", page, search],
    queryFn: () => listReading(page, search, 20),
  });
}

export function useReadingDetail(id: string) {
  return useQuery({
    queryKey: ["reading", id],
    queryFn: () => getReadingDetail(id),
    enabled: !!id,
  });
}
```

---

### Task 6: ReadingContent + FuriganaToggle components

**Files:**
- Create: `src/features/reading/components/ReadingContent.tsx`
- Create: `src/features/reading/components/FuriganaToggle.tsx`

---

- [ ] **Step 1: Create FuriganaToggle.tsx**

```tsx
import { Button } from "@/components/ui/button";

interface Props {
  show: boolean;
  onToggle: () => void;
}

export default function FuriganaToggle({ show, onToggle }: Props) {
  return (
    <Button variant="outline" size="sm" onClick={onToggle}>
      {show ? "振り仮名: 表示" : "振り仮名: 非表示"}
    </Button>
  );
}
```

---

- [ ] **Step 2: Create ReadingContent.tsx**

Renders paragraphs with ruby annotations. Supports click-to-lookup.

```tsx
import { useCallback } from "react";
import { Segment } from "@/services/api";

interface Props {
  segmentsJson: string;
  showFurigana: boolean;
  onWordClick: (word: string) => void;
}

export default function ReadingContent({ segmentsJson, showFurigana, onWordClick }: Props) {
  const paragraphs: Segment[][] = JSON.parse(segmentsJson);

  return (
    <div className={`space-y-4 leading-relaxed text-base ${showFurigana ? "" : "hide-furigana"}`}>
      <style>{`.hide-furigana rt { display: none; }`}</style>
      {paragraphs.map((para, pi) => (
        <p key={pi} className="text-justify">
          {para.map((seg, si) => {
            if (seg.k && seg.f) {
              return (
                <ruby
                  key={si}
                  className="cursor-pointer decoration-dotted underline-offset-2 hover:bg-yellow-100 dark:hover:bg-yellow-900/30"
                  onClick={() => onWordClick(seg.t)}
                >
                  {seg.t}<rt>{seg.f}</rt>
                </ruby>
              );
            }
            return <span key={si}>{seg.t}</span>;
          })}
        </p>
      ))}
    </div>
  );
}
```

---

### Task 7: WordPopover component

**Files:**
- Create: `src/features/reading/components/WordPopover.tsx`

---

- [ ] **Step 1: Create WordPopover.tsx**

```tsx
import { useEffect, useRef } from "react";
import { useQuery } from "@tanstack/react-query";
import { lookupWord } from "@/services/api";
import { Card } from "@/components/ui/card";

interface Props {
  word: string;
  position: { x: number; y: number };
  onClose: () => void;
}

export default function WordPopover({ word, position, onClose }: Props) {
  const ref = useRef<HTMLDivElement>(null);
  const { data, isLoading } = useQuery({
    queryKey: ["lookup", word],
    queryFn: () => lookupWord(word),
  });

  useEffect(() => {
    const handler = (e: MouseEvent) => {
      if (ref.current && !ref.current.contains(e.target as Node)) onClose();
    };
    const escHandler = (e: KeyboardEvent) => {
      if (e.key === "Escape") onClose();
    };
    document.addEventListener("mousedown", handler);
    document.addEventListener("keydown", escHandler);
    return () => {
      document.removeEventListener("mousedown", handler);
      document.removeEventListener("keydown", escHandler);
    };
  }, [onClose]);

  return (
    <div
      ref={ref}
      className="fixed z-50"
      style={{ left: position.x, top: position.y + 8 }}
    >
      <Card className="w-72 p-4 shadow-xl">
        {isLoading ? (
          <p className="text-sm text-muted-foreground">查询中...</p>
        ) : data ? (
          <div className="space-y-1">
            <p className="text-lg font-bold">{data.word}</p>
            <p className="text-sm text-muted-foreground">{data.reading}</p>
            <p className="text-sm">{data.meaning}</p>
            {data.example && (
              <p className="mt-2 text-xs italic text-muted-foreground">{data.example}</p>
            )}
          </div>
        ) : (
          <p className="text-sm text-muted-foreground">未在词汇库中找到</p>
        )}
      </Card>
    </div>
  );
}
```

---

### Task 8: QuizSection + QuizQuestion components

**Files:**
- Create: `src/features/reading/components/QuizSection.tsx`
- Create: `src/features/reading/components/QuizQuestion.tsx`

---

- [ ] **Step 1: Create QuizQuestion.tsx**

```tsx
import { useState } from "react";
import { Button } from "@/components/ui/button";
import { Question } from "@/services/api";

interface Props {
  question: Question;
  index: number;
}

export default function QuizQuestion({ question, index }: Props) {
  const [selected, setSelected] = useState<number | null>(null);
  const answered = selected !== null;
  const isCorrect = selected === question.correct_index;

  return (
    <div className="rounded-lg border p-4">
      <p className="mb-3 font-medium">
        {index + 1}. {question.question}
      </p>
      <div className="space-y-2">
        {question.options.map((opt, oi) => {
          let variant: "outline" | "default" | "secondary" | "destructive" = "outline";
          if (answered) {
            if (oi === question.correct_index) variant = "default";
            else if (oi === selected) variant = "destructive";
          }
          return (
            <Button
              key={oi}
              variant={variant}
              className="w-full justify-start text-left"
              disabled={answered}
              onClick={() => setSelected(oi)}
            >
              {opt}
            </Button>
          );
        })}
      </div>
      {answered && (
        <div className={`mt-3 rounded-md p-3 text-sm ${isCorrect ? "bg-green-100 text-green-800 dark:bg-green-900/30 dark:text-green-300" : "bg-red-100 text-red-800 dark:bg-red-900/30 dark:text-red-300"}`}>
          <p className="font-medium">{isCorrect ? "✓ 正解！" : "✗ 不正解"}</p>
          <p className="mt-1">{question.explanation}</p>
        </div>
      )}
    </div>
  );
}
```

---

- [ ] **Step 2: Create QuizSection.tsx**

```tsx
import { useMemo } from "react";
import { Question } from "@/services/api";
import QuizQuestion from "./QuizQuestion";
import { Separator } from "@/components/ui/separator";

interface Props {
  questionsJson: string | null;
}

export default function QuizSection({ questionsJson }: Props) {
  const questions: Question[] = useMemo(() => {
    if (!questionsJson) return [];
    try { return JSON.parse(questionsJson); } catch { return []; }
  }, [questionsJson]);

  if (questions.length === 0) return null;

  return (
    <div className="mt-8 space-y-4">
      <Separator />
      <h2 className="text-xl font-bold">読解問題</h2>
      {questions.map((q, i) => (
        <QuizQuestion key={q.id} question={q} index={i} />
      ))}
    </div>
  );
}
```

---

### Task 9: ReadingListPage

**Files:**
- Create: `src/features/reading/pages/ReadingListPage.tsx`

---

- [ ] **Step 1: Create ReadingListPage.tsx**

Follows GrammarListPage pattern exactly:

```tsx
import { useState } from "react";
import { useNavigate } from "react-router-dom";
import { Input } from "@/components/ui/input";
import { Card, CardContent } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Skeleton } from "@/components/ui/skeleton";
import { useReadingList } from "../hooks/useReading";

export default function ReadingListPage() {
  const [search, setSearch] = useState("");
  const [page, setPage] = useState(1);
  const { data, isLoading } = useReadingList(page, search || undefined);
  const navigate = useNavigate();

  return (
    <div className="space-y-4">
      <h1 className="text-2xl font-bold">阅读</h1>
      <Input
        placeholder="搜索文章..."
        value={search}
        onChange={(e) => { setSearch(e.target.value); setPage(1); }}
      />
      {isLoading ? (
        <div className="space-y-3">
          {Array.from({ length: 4 }).map((_, i) => (
            <Skeleton key={i} className="h-20 w-full" />
          ))}
        </div>
      ) : data && data.items.length > 0 ? (
        <div className="space-y-3">
          {data.items.map((article) => (
            <Card
              key={article.id}
              className="cursor-pointer transition-colors hover:bg-muted/50"
              onClick={() => navigate(`/reading/${article.id}`)}
            >
              <CardContent className="flex items-center justify-between p-4">
                <div>
                  <p className="font-medium">{article.title}</p>
                </div>
                <Badge variant="secondary">{article.level}</Badge>
              </CardContent>
            </Card>
          ))}
        </div>
      ) : (
        <div className="py-12 text-center text-muted-foreground">
          {search ? "没有找到匹配的文章" : "暂无阅读文章"}
        </div>
      )}
    </div>
  );
}
```

---

### Task 10: ReadingDetailPage

**Files:**
- Create: `src/features/reading/pages/ReadingDetailPage.tsx`

---

- [ ] **Step 1: Create ReadingDetailPage.tsx**

Combines ReadingContent, FuriganaToggle, WordPopover, QuizSection:

```tsx
import { useState, useCallback } from "react";
import { useParams, useNavigate } from "react-router-dom";
import { Button } from "@/components/ui/button";
import { Skeleton } from "@/components/ui/skeleton";
import { useReadingDetail } from "../hooks/useReading";
import ReadingContent from "../components/ReadingContent";
import FuriganaToggle from "../components/FuriganaToggle";
import WordPopover from "../components/WordPopover";
import QuizSection from "../components/QuizSection";

export default function ReadingDetailPage() {
  const { id } = useParams<{ id: string }>();
  const { data, isLoading, error } = useReadingDetail(id!);
  const navigate = useNavigate();
  const [showFurigana, setShowFurigana] = useState(true);
  const [popover, setPopover] = useState<{ word: string; x: number; y: number } | null>(null);

  const handleWordClick = useCallback((word: string, e: React.MouseEvent) => {
    const rect = (e.target as HTMLElement).getBoundingClientRect();
    setPopover({ word, x: rect.left, y: rect.bottom });
  }, []);

  if (isLoading) {
    return (
      <div className="space-y-4">
        <Skeleton className="h-8 w-32" />
        <Skeleton className="h-6 w-48" />
        {Array.from({ length: 6 }).map((_, i) => (
          <Skeleton key={i} className="h-4 w-full" />
        ))}
      </div>
    );
  }

  if (error || !data) {
    return (
      <div className="flex flex-col items-center gap-3 py-12">
        <p className="text-destructive">文章不存在</p>
        <Button variant="outline" onClick={() => navigate("/reading")}>返回列表</Button>
      </div>
    );
  }

  return (
    <div className="mx-auto max-w-3xl space-y-6">
      <div className="flex items-center justify-between">
        <Button variant="ghost" onClick={() => navigate("/reading")}>← 返回列表</Button>
        <FuriganaToggle
          show={showFurigana}
          onToggle={() => setShowFurigana((v) => !v)}
        />
      </div>

      <h1 className="text-2xl font-bold">{data.title}</h1>

      <ReadingContent
        segmentsJson={data.segments}
        showFurigana={showFurigana}
        onWordClick={(word) => {
          // Create a synthetic event for positioning
          const el = document.querySelector(`ruby:has(rt)`);
          if (el) {
            const rect = el.getBoundingClientRect();
            setPopover({ word, x: rect.left, y: rect.bottom });
          } else {
            setPopover({ word, x: window.innerWidth / 2, y: 200 });
          }
        }}
      />

      {popover && (
        <WordPopover
          word={popover.word}
          position={{ x: popover.x, y: popover.y }}
          onClose={() => setPopover(null)}
        />
      )}

      <QuizSection questionsJson={data.questions} />
    </div>
  );
}
```

Wait, the onWordClick handler is wrong. Let me fix it. The ReadingContent component already handles the click event with `onClick={() => onWordClick(seg.t)}`. But we need the mouse position to position the popover. Let me adjust:

In ReadingContent, change the click handler to pass the event:

Actually no. The WordPopover uses a fixed position state. Let me simplify: the `ReadingContent` component already has access to the click event via `onClick`. Let me change the approach slightly — the onWordClick callback in ReadingContent should get the click event.

Let me revise ReadingContent's click handler:

```tsx
onClick={(e) => onWordClick(seg.t, e)}
```

And the prop type:
```tsx
onWordClick: (word: string, e: React.MouseEvent) => void;
```

Then in ReadingDetailPage:
```tsx
const handleWordClick = useCallback((word: string, e: React.MouseEvent) => {
  const rect = (e.target as HTMLElement).getBoundingClientRect();
  setPopover({ word, x: rect.left, y: rect.bottom });
}, []);
```

OK let me rewrite the page properly:

```tsx
import { useState, useCallback } from "react";
import { useParams, useNavigate } from "react-router-dom";
import { Button } from "@/components/ui/button";
import { Skeleton } from "@/components/ui/skeleton";
import { useReadingDetail } from "../hooks/useReading";
import ReadingContent from "../components/ReadingContent";
import FuriganaToggle from "../components/FuriganaToggle";
import WordPopover from "../components/WordPopover";
import QuizSection from "../components/QuizSection";

export default function ReadingDetailPage() {
  const { id } = useParams<{ id: string }>();
  const { data, isLoading, error } = useReadingDetail(id!);
  const navigate = useNavigate();
  const [showFurigana, setShowFurigana] = useState(true);
  const [popover, setPopover] = useState<{ word: string; x: number; y: number } | null>(null);

  const handleWordClick = useCallback((word: string, e: React.MouseEvent) => {
    const rect = (e.target as HTMLElement).getBoundingClientRect();
    setPopover({ word, x: rect.left, y: rect.bottom });
  }, []);

  if (isLoading) {
    return (
      <div className="space-y-4">
        <Skeleton className="h-8 w-32" />
        <Skeleton className="h-6 w-48" />
        {Array.from({ length: 6 }).map((_, i) => (
          <Skeleton key={i} className="h-4 w-full" />
        ))}
      </div>
    );
  }

  if (error || !data) {
    return (
      <div className="flex flex-col items-center gap-3 py-12">
        <p className="text-destructive">文章不存在</p>
        <Button variant="outline" onClick={() => navigate("/reading")}>返回列表</Button>
      </div>
    );
  }

  return (
    <div className="mx-auto max-w-3xl space-y-6">
      <div className="flex items-center justify-between">
        <Button variant="ghost" onClick={() => navigate("/reading")}>← 返回列表</Button>
        <FuriganaToggle
          show={showFurigana}
          onToggle={() => setShowFurigana((v) => !v)}
        />
      </div>

      <h1 className="text-2xl font-bold">{data.title}</h1>

      <ReadingContent
        segmentsJson={data.segments}
        showFurigana={showFurigana}
        onWordClick={handleWordClick}
      />

      {popover && (
        <WordPopover
          word={popover.word}
          position={{ x: popover.x, y: popover.y }}
          onClose={() => setPopover(null)}
        />
      )}

      <QuizSection questionsJson={data.questions} />
    </div>
  );
}
```

OK good. Now let me write all tasks.

Wait, I need to also update the ReadingContent component to accept `onWordClick(word, event)`:

```tsx
interface Props {
  segmentsJson: string;
  showFurigana: boolean;
  onWordClick: (word: string, e: React.MouseEvent) => void;
}
```

And the handler:
```tsx
onClick={(e) => onWordClick(seg.t, e)}
```

Let me rewrite Task 6's ReadingContent accordingly.

---

### Task 11: Routes + Dashboard update

**Files:**
- Modify: `src/routes/index.tsx`
- Modify: `src-tauri/src/commands/dashboard.rs`

---

- [ ] **Step 1: Add reading routes to routes/index.tsx**

```tsx
import ReadingListPage from "@/features/reading/pages/ReadingListPage";
import ReadingDetailPage from "@/features/reading/pages/ReadingDetailPage";

// In children array, add:
{ path: "reading", element: <ReadingListPage /> },
{ path: "reading/:id", element: <ReadingDetailPage /> },
```

- [ ] **Step 2: Add reading progress to dashboard.rs**

After kanji progress (~line 66), add:

```rust
    let reading_total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM reading")
        .fetch_one(pool)
        .await
        .map_err(|e| e.to_string())?;
    // Reading has no flashcards, so completed = 0 for now
    // (could track via bookmarks in future)
    let reading_done: i64 = 0;
```

Update `module_progress`:

```rust
    let module_progress = vec![
        ModuleProgress { module: "vocabulary".into(), completed: vocab_done, total: vocab_total },
        ModuleProgress { module: "grammar".into(), completed: grammar_done, total: grammar_total },
        ModuleProgress { module: "kanji".into(), completed: kanji_done, total: kanji_total },
        ModuleProgress { module: "reading".into(), completed: reading_done, total: reading_total },
    ];
```

---

### Task 12: Build + verify

- [ ] **Step 1: Full build check**

```bash
cd src-tauri && cargo build 2>&1
```

Expected: Compilation succeeds with no errors

- [ ] **Step 2: Frontend type check**

```bash
cd /home/joshua/vibe/N3LearningOS && npx tsc --noEmit 2>&1
```

Expected: No TypeScript errors

- [ ] **Step 3: Test import**

Run the app. Expected: "Import: ... X reading, ..." in logs.

- [ ] **Step 4: Final commit**

```bash
git add -A && git commit -m "feat: Phase 3 reading module — article reader, furigana toggle, word lookup, comprehension quiz

- Migration 003: reading table with title, segments (JSON), questions (JSON)
- Rust models: Reading, Segment, Question structs
- Rust commands: list_reading, get_reading_detail, lookup_word
- Reading importer with manifest-driven pipeline integration
- 15 pre-bundled N3 reading articles with embedded furigana
- Frontend: ReadingListPage, ReadingDetailPage, ReadingContent (ruby render)
- FuriganaToggle component for show/hide control
- WordPopover for click-to-lookup vocabulary
- QuizSection + QuizQuestion for comprehension assessment
- Dashboard updated with reading module progress
- No flashcards for reading (quiz-based assessment)

Co-Authored-By: Claude Opus 4.8 <noreply@anthropic.com>"
```
