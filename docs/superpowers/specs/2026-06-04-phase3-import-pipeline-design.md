# Import Pipeline — Design Spec

> **Phase 3 Part 3:** CSV / JSON / APKG user import for all 5 content types.

**Goal:** Allow users to import their own vocabulary, grammar, kanji, reading, and listening content into N3LearningOS from local files.

**Architecture:** Each module page gets an ImportButton → Tauri native file dialog → backend parses & writes to DB → result dialog. No file storage outside DB.

**Tech Stack:** Rust (`zip` + `rusqlite` for APKG), Tauri `dialog` plugin, shadcn/ui Dialog + Button

---

## 1. Supported Formats & Content Types

| Format | Vocabulary | Grammar | Kanji | Reading | Listening |
|--------|-----------|---------|-------|---------|-----------|
| CSV    | ✅        | ❌      | ❌    | ❌      | ❌        |
| JSON   | ✅        | ✅      | ✅    | ✅      | ✅        |
| APKG   | ✅        | ✅      | ✅    | ❌      | ❌        |

### CSV (vocabulary only)
- UTF-8 encoding
- First row is header
- Columns: `word`, `reading`, `meaning`, `example` (optional), `tags` (optional, pipe-delimited)
- Example: `勉強,べんきょう,study,私は日本語を勉強します, N3|vocab`

### JSON (all types)
- Array of objects matching the same schema as bundled content JSONs
- Each object field set matches the corresponding model's import struct
- Reading: `{ title, segments, questions?, source?, level? }`
- Listening: `{ title, audio_file?, transcript, questions?, source?, level? }` (audio_file is path to existing MP3 in content/audio/ — user must place it manually, or omit for transcript-only import)

### APKG (vocabulary / grammar / kanji)
- Standard Anki `.apkg` zip format
- Contains `collection.anki21` SQLite database
- Field mapping uses heuristic matching on model name and field names
- Unrecognized models default to vocabulary

---

## 2. Backend Architecture

### 2.1 Tauri Commands

Five new commands, one per content type:

```rust
#[tauri::command]
async fn import_vocabulary_from_file(
    file_path: String,
    mode: ImportMode,
) -> Result<ImportFileResult, String>

#[tauri::command]
async fn import_grammar_from_file(
    file_path: String,
    mode: ImportMode,
) -> Result<ImportFileResult, String>

#[tauri::command]
async fn import_kanji_from_file(
    file_path: String,
    mode: ImportMode,
) -> Result<ImportFileResult, String>

#[tauri::command]
async fn import_reading_from_file(
    file_path: String,
    mode: ImportMode,
) -> Result<ImportFileResult, String>

#[tauri::command]
async fn import_listening_from_file(
    file_path: String,
    mode: ImportMode,
) -> Result<ImportFileResult, String>
```

### 2.2 Shared Types

```rust
#[derive(Serialize, Deserialize)]
pub enum ImportMode {
    Append,    // INSERT OR IGNORE — skip duplicates
    Overwrite, // INSERT OR REPLACE — overwrite on conflict
}

#[derive(Serialize, Deserialize)]
pub struct ImportFileResult {
    pub imported: usize,
    pub overwritten: usize,
    pub errors: Vec<String>,
    pub total_in_file: usize,
}
```

### 2.3 Dedup / Conflict Detection

| Content Type | Conflict Key | SQL Clause |
|-------------|-------------|------------|
| vocabulary  | `word`      | `INSERT OR {IGNORE / REPLACE} INTO vocabulary (word, ...)` |
| grammar     | `pattern`   | `INSERT OR {IGNORE / REPLACE} INTO grammar (pattern, ...)` |
| kanji       | `character` | `INSERT OR {IGNORE / REPLACE} INTO kanji (character, ...)` |
| reading     | `title`     | `INSERT OR {IGNORE / REPLACE} INTO reading (title, ...)` |
| listening   | `title`     | `INSERT OR {IGNORE / REPLACE} INTO listening (title, ...)` |

### 2.4 Format Detection

Based on file extension:
- `.csv` → CSV parser (vocabulary only, error for other types)
- `.json` → JSON parser (all types)
- `.apkg` → APKG parser (vocabulary/grammar/kanji only)

### 2.5 Parser Modules

Each format gets its own Rust module under `src-tauri/src/importers/`:

**`csv_parser.rs`** — Uses `csv` crate. Reads header row to map columns. Validates required fields (`word`, `reading`, `meaning`). Returns `Vec<Vocabulary>`.

**`json_parser.rs`** — Uses `serde_json`. Each content type has its own deserialization function. Validates required fields per type. Returns `Vec<T>`.

**`apkg_parser.rs`** — Uses `zip` + `rusqlite`:
1. Extract zip to temp dir
2. Open `collection.anki21`
3. Read `col` table → parse `models` JSON to get field name → index mapping per model
4. Query `notes` table, split `flds` by `\x1f`, map to struct fields by index
5. Heuristic content-type detection:
   - Model name contains "kanji"/"漢字"/"汉字" → kanji
   - Model name contains "grammar"/"文法"/"语法" → grammar
   - Default → vocabulary
6. Clean up temp dir

### 2.6 New Cargo Dependencies

- `csv` — CSV parsing
- `zip` — APKG extraction
- `tempfile` — temp dir for APKG extraction
- `rusqlite` — reading APKG's `collection.anki21` (used independently of app's SQLite via `sqlx`)

---

## 3. Frontend Architecture

### 3.1 Directory Structure

```
src/features/import/
├── components/
│   ├── ImportButton.tsx        # Icon button + file dialog trigger
│   └── ImportResultDialog.tsx   # Import result display dialog
└── hooks/
    └── useImport.ts            # Import mutation hook
```

### 3.2 ImportButton Component

```tsx
interface ImportButtonProps {
  contentType: "vocabulary" | "grammar" | "kanji" | "reading" | "listening";
  onImportComplete: () => void;
}
```

**States:**
- **Default:** Icon button with tooltip "导入"
- **Loading:** Disabled + spinner during import
- **Error:** Toast on dialog open error (permission denied, etc.)

**Behavior:**
1. Click → `open()` from `@tauri-apps/plugin-dialog` with content-type-specific filters
2. User cancels → no-op
3. File selected → show ImportMode dialog (Append / Overwrite)
4. User selects mode → `invoke()` import command
5. Complete → show ImportResultDialog → `onImportComplete()` callback

### 3.3 ImportResultDialog Component

Props: `{ result: ImportFileResult | null, open: boolean, onOpenChange: (open: boolean) => void }`

Uses shadcn Dialog.

**UI layout:**
```
┌─ 导入结果 ──────────────────┐
│                             │
│  ✓ 成功导入: {imported}       │
│  📝 覆盖更新: {overwritten}   │
│  ❌ 错误: {errors.length}     │
│    • 第3行: word 字段不能为空  │
│    • 第7行: reading 格式无效  │
│                             │
│        [ 确定 ]              │
└─────────────────────────────┘
```

**States:**
- **null / closed** — hidden
- **all success, no errors** — green checkmark, brief summary
- **partial errors** — yellow warning, error list collapsed under expandable section
- **all failed** — red X, error list expanded

### 3.4 useImport Hook

```ts
function useImport(contentType: string) {
  return useMutation({
    mutationFn: (params: { filePath: string; mode: "Append" | "Overwrite" }) =>
      invoke(`import_${contentType}_from_file`, params),
  });
}
```

### 3.5 Page Modifications

Each of 5 ListPages adds ImportButton in the header area (next to search bar or title):

- `VocabListPage.tsx`
- `GrammarListPage.tsx`
- `KanjiListPage.tsx`
- `ReadingListPage.tsx`
- `ListeningListPage.tsx`

---

## 4. Data Flow

```
[ImportButton click]
       ↓
[tauri-plugin-dialog: open()] → user picks file
       ↓
[ModeSelector dialog] → user picks Append/Overwrite
       ↓
[invoke("import_vocabulary_from_file", { filePath, mode })]
       ↓
[Backend: detect format by extension]
  ├── .csv  → csv_parser → Vec<Vocabulary>
  ├── .json → json_parser → Vec<T>
  └── .apkg → apkg_parser → Vec<Vocabulary|Grammar|Kanji>
       ↓
[Backend: INSERT OR {IGNORE|REPLACE} INTO ...]
       ↓
[Backend: return ImportFileResult]
       ↓
[Frontend: ImportResultDialog] → show result
       ↓
[onImportComplete] → refetch list
```

---

## 5. States & Edge Cases

### Loading
- ImportButton shows spinner, disabled during import
- Progress: indeterminate spinner (no streaming progress from backend)

### Empty
- No file selected → no-op
- File has 0 valid rows → "文件中没有有效数据"

### Error States
- **File not found** — dialog.open error (OS-level)
- **Unsupported format** — ".xlsx 不支持，请使用 CSV/JSON/APKG"
- **CSV for non-vocabulary** — "CSV 仅支持词汇导入"
- **APKG for reading/listening** — "APKG 不支持阅读/听力导入"
- **Parse error (JSON)** — "JSON 格式无效: {details}" with line info
- **Parse error (CSV)** — "CSV 第 {N} 行: {details}"
- **APKG missing collection.anki21** — "无效的 APKG 文件"
- **Content-type detection failed (APKG)** — default to vocabulary
- **Duplicate entries in file** — reported as skipped/overwritten count
- **DB write error** — "数据库写入失败: {details}"

### Duplicate Handling
- Append mode: INSERT OR IGNORE, count as skipped
- Overwrite mode: INSERT OR REPLACE, count as overwritten
- Report counts separately in ImportFileResult

---

## 6. API Layer

New functions in `src/services/api.ts`:

```typescript
export interface ImportFileResult {
  imported: number;
  overwritten: number;
  errors: string[];
  total_in_file: number;
}

export function importVocabularyFromFile(
  filePath: string,
  mode: "Append" | "Overwrite"
): Promise<ImportFileResult> {
  return invoke("import_vocabulary_from_file", { filePath, mode });
}
// ... same pattern for grammar, kanji, reading, listening
```

---

## 7. Route & Navigation

No new routes. Import functionality is embedded into existing ListPages.

---

## 8. Out of Scope

- Streaming / progress reporting for large imports
- Drag-and-drop import
- Import history / undo import
- Bulk export
- Import from cloud services (Google Sheets, AnkiWeb)
- Automatic audio file copying for listening import
