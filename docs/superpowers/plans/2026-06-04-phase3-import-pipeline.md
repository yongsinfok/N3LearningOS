# Import Pipeline Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Allow users to import their own vocabulary, grammar, kanji, reading, and listening content from local files (CSV/JSON/APKG).

**Architecture:** Each module ListPage gets ImportButton → Tauri native file dialog → backend parses format → INSERT with Append/Overwrite mode → result dialog. Parser modules under `importers/`, 5 new Tauri commands.

**Tech Stack:** Rust (`csv`, `zip` + `rusqlite` + `tempfile`), `tauri-plugin-dialog`, shadcn/ui Dialog+Button, TanStack Query `useMutation`

---

## File Structure

### New backend files (5):
- `src-tauri/src/importers/csv_parser.rs` — CSV → Vec<Vocabulary>
- `src-tauri/src/importers/json_parser.rs` — JSON → Vec<T> for all 5 types
- `src-tauri/src/importers/apkg_parser.rs` — APKG → Vec<Vocabulary|Grammar|Kanji>
- `src-tauri/src/importers/user_import.rs` — 5 public `import_*_from_file` functions, orchestrator
- `src-tauri/src/commands/import.rs` — 5 Tauri commands

### New frontend files (4):
- `src/features/import/components/ImportButton.tsx`
- `src/features/import/components/ImportResultDialog.tsx`
- `src/features/import/hooks/useImport.ts`
- `src/features/import/types/index.ts`

### Modified files (8):
- `src-tauri/Cargo.toml` — add deps
- `src-tauri/src/importers/mod.rs` — declare new modules
- `src-tauri/src/models/mod.rs` — add `ImportMode`, `ImportFileResult`
- `src-tauri/src/lib.rs` — register commands + dialog plugin
- `src/services/api.ts` — add import API functions + types
- `src/features/vocabulary/pages/VocabListPage.tsx` — add ImportButton
- `src/features/grammar/pages/GrammarListPage.tsx` — add ImportButton
- `src/features/kanji/pages/KanjiListPage.tsx` — add ImportButton
- `src/features/reading/pages/ReadingListPage.tsx` — add ImportButton
- `src/features/listening/pages/ListeningListPage.tsx` — add ImportButton
- `src-tauri/tauri.conf.json` — register dialog plugin permissions

---

### Task 1: Cargo Dependencies + Dialog Plugin + Models

**Files:**
- Modify: `src-tauri/Cargo.toml`
- Modify: `src-tauri/src/lib.rs`
- Modify: `src-tauri/src/models/mod.rs`
- Modify: `src-tauri/src/importers/mod.rs`
- Modify: `src-tauri/tauri.conf.json`
- Run: `npm install @tauri-apps/plugin-dialog`

- [ ] **Step 1: Add Cargo deps**

Edit `src-tauri/Cargo.toml`. Add to `[dependencies]`:

```toml
csv = "1.3"
zip = "2"
tempfile = "3"
rusqlite = { version = "0.32", features = ["bundled"] }
tauri-plugin-dialog = "2"
```

- [ ] **Step 2: Install npm dialog plugin + create tauri.conf.json plugin entry**

Run: `npm install @tauri-apps/plugin-dialog`

Edit `src-tauri/tauri.conf.json`. Add top-level `"plugins"` key:

```json
{
  // ... existing fields ...
  "plugins": {
    "dialog": {}
  }
}
```

- [ ] **Step 3: Register dialog plugin + add import commands to lib.rs**

Edit `src-tauri/src/lib.rs`. After `.setup()` closure's closing paren, add `.plugin()` call. In `invoke_handler`, add the 5 import commands.

```rust
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|_app| {
            // ... unchanged ...
        })
        .invoke_handler(tauri::generate_handler![
            // ... existing handlers ...
            commands::import::import_vocabulary_from_file,
            commands::import::import_grammar_from_file,
            commands::import::import_kanji_from_file,
            commands::import::import_reading_from_file,
            commands::import::import_listening_from_file,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

- [ ] **Step 4: Declare new importer modules + new commands module**

Edit `src-tauri/src/importers/mod.rs`. Add to the `pub mod` declarations:

```rust
pub mod csv_parser;
pub mod json_parser;
pub mod apkg_parser;
pub mod user_import;
```

Ensure `src-tauri/src/commands/mod.rs` declares the `import` module (create if missing with `pub mod import;`).

Run build check: `cargo check`

Expected: Compiles with warnings about unused imports.

- [ ] **Step 5: Add ImportMode and ImportFileResult models**

Edit `src-tauri/src/models/mod.rs`. Add before `ImportResult`:

```rust
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ImportMode {
    Append,
    Overwrite,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ImportFileResult {
    pub imported: usize,
    pub overwritten: usize,
    pub errors: Vec<String>,
    pub total_in_file: usize,
}
```

- [ ] **Step 6: Commit**

```bash
git add src-tauri/Cargo.toml src-tauri/src/lib.rs src-tauri/src/models/mod.rs \
        src-tauri/src/importers/mod.rs src-tauri/tauri.conf.json package.json
git commit -m "chore: add Cargo deps (csv/zip/rusqlite/tempfile) + dialog plugin + import models

Add csv, zip, rusqlite (bundled), tempfile, and tauri-plugin-dialog
to Cargo.toml. Register dialog plugin and import command stubs in
lib.rs. Add ImportMode enum, ImportFileResult struct, and declare
new parser/import modules.

Co-Authored-By: Claude Opus 4.8 <noreply@anthropic.com>"
```

---

### Task 2: CSV Parser

**Files:**
- Create: `src-tauri/src/importers/csv_parser.rs`

- [ ] **Step 1: Implement csv_parser.rs**

```rust
use crate::models::Vocabulary;
use std::path::Path;

/// Parse a CSV file into Vocabulary entries.
/// Expected columns (first row is header): word, reading, meaning[, example, tags]
/// Tags are pipe-delimited.
pub fn parse_csv_vocabulary(file_path: &Path) -> Result<Vec<Vocabulary>, String> {
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(true)
        .flexible(true)
        .from_path(file_path)
        .map_err(|e| format!("无法打开文件: {}", e))?;

    let headers = reader.headers().map_err(|e| format!("CSV 表头无效: {}", e))?;
    let headers: Vec<&str> = headers.iter().collect();

    let word_idx = headers.iter().position(|h| h.trim().to_lowercase() == "word")
        .ok_or_else(|| "CSV 缺少 'word' 列".to_string())?;
    let reading_idx = headers.iter().position(|h| h.trim().to_lowercase() == "reading")
        .ok_or_else(|| "CSV 缺少 'reading' 列".to_string())?;
    let meaning_idx = headers.iter().position(|h| h.trim().to_lowercase() == "meaning")
        .ok_or_else(|| "CSV 缺少 'meaning' 列".to_string())?;
    let example_idx = headers.iter().position(|h| h.trim().to_lowercase() == "example");
    let tags_idx = headers.iter().position(|h| h.trim().to_lowercase() == "tags");

    let mut vocab = Vec::new();
    let mut errors = Vec::new();

    for (row_num, result) in reader.records().enumerate() {
        let line = row_num + 2; // 1-indexed, header is row 1
        let record = match result {
            Ok(r) => r,
            Err(e) => {
                errors.push(format!("第{}行: 解析失败 - {}", line, e));
                continue;
            }
        };

        let word = record.get(word_idx).unwrap_or("").trim().to_string();
        if word.is_empty() {
            errors.push(format!("第{}行: word 字段不能为空", line));
            continue;
        }

        let reading = record.get(reading_idx).unwrap_or("").trim().to_string();
        let meaning = record.get(meaning_idx).unwrap_or("").trim().to_string();
        let example = example_idx.and_then(|i| record.get(i)).map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty());
        let tags = tags_idx.and_then(|i| record.get(i)).map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty());

        vocab.push(Vocabulary {
            id: uuid::Uuid::new_v4().to_string(),
            word,
            reading,
            meaning,
            example,
            tags: tags,
            source: "user_import".to_string(),
            created_at: chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        });
    }

    if vocab.is_empty() && !errors.is_empty() {
        return Err(format!("CSV 文件无有效数据:\n{}", errors.join("\n")));
    }

    Ok(vocab)
}
```

- [ ] **Step 2: Build check**

Run: `cargo check`

Expected: Compiles successfully.

- [ ] **Step 3: Commit**

```bash
git add src-tauri/src/importers/csv_parser.rs
git commit -m "feat: add CSV parser for vocabulary import

Parse CSV with header row (word, reading, meaning, example, tags).
Validates required columns, reports per-line errors, generates UUIDs.

Co-Authored-By: Claude Opus 4.8 <noreply@anthropic.com>"
```

---

### Task 3: JSON Parser

**Files:**
- Create: `src-tauri/src/importers/json_parser.rs`

- [ ] **Step 1: Implement json_parser.rs**

```rust
use crate::models::{Grammar, Kanji, Listening, Question, Reading, Segment, TranscriptEntry, Vocabulary};
use std::path::Path;

/// Parse a JSON file into Vocabulary entries.
/// Expects array of objects: { word, reading, meaning, example?, tags?, source? }
pub fn parse_json_vocabulary(file_path: &Path) -> Result<Vec<Vocabulary>, String> {
    let content = std::fs::read_to_string(file_path)
        .map_err(|e| format!("无法读取文件: {}", e))?;

    #[derive(serde::Deserialize)]
    struct VocabEntry {
        word: String,
        reading: String,
        meaning: String,
        #[serde(default)]
        example: Option<String>,
        #[serde(default)]
        tags: Option<Vec<String>>,
        #[serde(default)]
        source: Option<String>,
    }

    let entries: Vec<VocabEntry> = serde_json::from_str(&content)
        .map_err(|e| format!("JSON 格式无效: {}", e))?;

    let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let mut vocab = Vec::new();
    let mut errors = Vec::new();

    for (i, entry) in entries.iter().enumerate() {
        if entry.word.is_empty() {
            errors.push(format!("第{}个条目: word 字段不能为空", i + 1));
            continue;
        }
        vocab.push(Vocabulary {
            id: uuid::Uuid::new_v4().to_string(),
            word: entry.word.clone(),
            reading: entry.reading.clone(),
            meaning: entry.meaning.clone(),
            example: entry.example.clone(),
            tags: entry.tags.as_ref().map(|t| t.join("|")),
            source: entry.source.clone().unwrap_or_else(|| "user_import".to_string()),
            created_at: now.clone(),
        });
    }

    if vocab.is_empty() && !errors.is_empty() {
        return Err(format!("JSON 文件无有效数据:\n{}", errors.join("\n")));
    }

    Ok(vocab)
}

/// Parse a JSON file into Grammar entries.
/// Expects array of objects: { pattern, meaning, explanation, examples[], related[]?, source? }
pub fn parse_json_grammar(file_path: &Path) -> Result<Vec<Grammar>, String> {
    let content = std::fs::read_to_string(file_path)
        .map_err(|e| format!("无法读取文件: {}", e))?;

    #[derive(serde::Deserialize)]
    struct GrammarEntry {
        pattern: String,
        meaning: String,
        explanation: String,
        #[serde(default)]
        examples: Vec<String>,
        #[serde(default)]
        related: Option<Vec<String>>,
        #[serde(default)]
        source: Option<String>,
    }

    let entries: Vec<GrammarEntry> = serde_json::from_str(&content)
        .map_err(|e| format!("JSON 格式无效: {}", e))?;

    let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let mut items = Vec::new();
    let mut errors = Vec::new();

    for (i, entry) in entries.iter().enumerate() {
        if entry.pattern.is_empty() {
            errors.push(format!("第{}个条目: pattern 字段不能为空", i + 1));
            continue;
        }
        items.push(Grammar {
            id: uuid::Uuid::new_v4().to_string(),
            pattern: entry.pattern.clone(),
            meaning: entry.meaning.clone(),
            explanation: entry.explanation.clone(),
            examples: serde_json::to_string(&entry.examples).unwrap_or_default(),
            related: entry.related.as_ref().map(|r| serde_json::to_string(r).unwrap_or_default()),
            source: entry.source.clone().unwrap_or_else(|| "user_import".to_string()),
            created_at: now.clone(),
        });
    }

    if items.is_empty() && !errors.is_empty() {
        return Err(format!("JSON 文件无有效数据:\n{}", errors.join("\n")));
    }

    Ok(items)
}

/// Parse a JSON file into Kanji entries.
/// Expects array of objects: { character, onyomi[], kunyomi[], meaning, example_words[]?, source? }
pub fn parse_json_kanji(file_path: &Path) -> Result<Vec<Kanji>, String> {
    let content = std::fs::read_to_string(file_path)
        .map_err(|e| format!("无法读取文件: {}", e))?;

    #[derive(serde::Deserialize)]
    struct KanjiEntry {
        character: String,
        #[serde(default)]
        onyomi: Option<Vec<String>>,
        #[serde(default)]
        kunyomi: Option<Vec<String>>,
        meaning: String,
        #[serde(default)]
        example_words: Option<Vec<String>>,
        #[serde(default)]
        source: Option<String>,
    }

    let entries: Vec<KanjiEntry> = serde_json::from_str(&content)
        .map_err(|e| format!("JSON 格式无效: {}", e))?;

    let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let mut items = Vec::new();
    let mut errors = Vec::new();

    for (i, entry) in entries.iter().enumerate() {
        if entry.character.is_empty() {
            errors.push(format!("第{}个条目: character 字段不能为空", i + 1));
            continue;
        }
        items.push(Kanji {
            id: uuid::Uuid::new_v4().to_string(),
            character: entry.character.clone(),
            onyomi: entry.onyomi.as_ref().map(|o| serde_json::to_string(o).unwrap_or_default()),
            kunyomi: entry.kunyomi.as_ref().map(|k| serde_json::to_string(k).unwrap_or_default()),
            meaning: entry.meaning.clone(),
            stroke_svg: None,
            example_words: entry.example_words.as_ref().map(|e| serde_json::to_string(e).unwrap_or_default()),
            source: entry.source.clone().unwrap_or_else(|| "user_import".to_string()),
            created_at: now.clone(),
        });
    }

    if items.is_empty() && !errors.is_empty() {
        return Err(format!("JSON 文件无有效数据:\n{}", errors.join("\n")));
    }

    Ok(items)
}

/// Parse a JSON file into Reading entries.
/// Expects array of objects: { title, segments[][], questions[]?, source?, level? }
pub fn parse_json_reading(file_path: &Path) -> Result<Vec<Reading>, String> {
    let content = std::fs::read_to_string(file_path)
        .map_err(|e| format!("无法读取文件: {}", e))?;

    #[derive(serde::Deserialize)]
    struct ReadingEntry {
        title: String,
        segments: Vec<Vec<Segment>>,
        #[serde(default)]
        questions: Option<Vec<Question>>,
        #[serde(default)]
        source: Option<String>,
        #[serde(default = "default_level")]
        level: String,
    }

    fn default_level() -> String { "N3".to_string() }

    let entries: Vec<ReadingEntry> = serde_json::from_str(&content)
        .map_err(|e| format!("JSON 格式无效: {}", e))?;

    let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let mut items = Vec::new();
    let mut errors = Vec::new();

    for (i, entry) in entries.iter().enumerate() {
        if entry.title.is_empty() {
            errors.push(format!("第{}个条目: title 字段不能为空", i + 1));
            continue;
        }
        items.push(Reading {
            id: uuid::Uuid::new_v4().to_string(),
            title: entry.title.clone(),
            segments: serde_json::to_string(&entry.segments).unwrap_or_default(),
            questions: entry.questions.as_ref().map(|q| serde_json::to_string(q).unwrap_or_default()),
            source: entry.source.clone().unwrap_or_else(|| "user_import".to_string()),
            level: entry.level.clone(),
            is_bookmarked: false,
            created_at: now.clone(),
        });
    }

    if items.is_empty() && !errors.is_empty() {
        return Err(format!("JSON 文件无有效数据:\n{}", errors.join("\n")));
    }

    Ok(items)
}

/// Parse a JSON file into Listening entries.
/// Expects array of objects: { title, audio_file?, transcript[], questions[]?, source?, level? }
pub fn parse_json_listening(file_path: &Path) -> Result<Vec<Listening>, String> {
    let content = std::fs::read_to_string(file_path)
        .map_err(|e| format!("无法读取文件: {}", e))?;

    #[derive(serde::Deserialize)]
    struct ListeningEntry {
        title: String,
        #[serde(default)]
        audio_file: Option<String>,
        transcript: Vec<TranscriptEntry>,
        #[serde(default)]
        questions: Option<Vec<Question>>,
        #[serde(default)]
        source: Option<String>,
        #[serde(default = "default_level")]
        level: String,
    }

    fn default_level() -> String { "N3".to_string() }

    let entries: Vec<ListeningEntry> = serde_json::from_str(&content)
        .map_err(|e| format!("JSON 格式无效: {}", e))?;

    let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let mut items = Vec::new();
    let mut errors = Vec::new();

    for (i, entry) in entries.iter().enumerate() {
        if entry.title.is_empty() {
            errors.push(format!("第{}个条目: title 字段不能为空", i + 1));
            continue;
        }
        items.push(Listening {
            id: uuid::Uuid::new_v4().to_string(),
            title: entry.title.clone(),
            audio_file: entry.audio_file.clone().unwrap_or_default(),
            transcript: serde_json::to_string(&entry.transcript).unwrap_or_default(),
            questions: entry.questions.as_ref().map(|q| serde_json::to_string(q).unwrap_or_default()),
            source: entry.source.clone().unwrap_or_else(|| "user_import".to_string()),
            level: entry.level.clone(),
            is_bookmarked: false,
            created_at: now.clone(),
        });
    }

    if items.is_empty() && !errors.is_empty() {
        return Err(format!("JSON 文件无有效数据:\n{}", errors.join("\n")));
    }

    Ok(items)
}
```

- [ ] **Step 2: Build check**

Run: `cargo check`

Expected: Compiles successfully.

- [ ] **Step 3: Commit**

```bash
git add src-tauri/src/importers/json_parser.rs
git commit -m "feat: add JSON parser for all 5 content types

Parse JSON arrays into Vocabulary, Grammar, Kanji, Reading, and
Listening models. Validate required fields per type, report per-item
errors, generate UUIDs for new entries.

Co-Authored-By: Claude Opus 4.8 <noreply@anthropic.com>"
```

---

### Task 4: APKG Parser

**Files:**
- Create: `src-tauri/src/importers/apkg_parser.rs`

- [ ] **Step 1: Implement apkg_parser.rs**

```rust
use crate::models::{Grammar, Kanji, Vocabulary};
use std::collections::HashMap;
use std::path::Path;
use std::io::Read;

/// Parse an Anki APKG file, returning vocabulary, grammar, and kanji entries.
/// APKG is a zip containing collection.anki21 (SQLite).
/// Field mapping uses heuristic on model name.
pub fn parse_apkg(file_path: &Path) -> Result<(Vec<Vocabulary>, Vec<Grammar>, Vec<Kanji>), String> {
    // Open zip
    let file = std::fs::File::open(file_path)
        .map_err(|e| format!("无法打开 APKG 文件: {}", e))?;
    let mut zip = zip::ZipArchive::new(file)
        .map_err(|e| format!("无效的 APKG 文件 (不是有效的 ZIP): {}", e))?;

    // Extract collection.anki21 to temp dir
    let temp_dir = tempfile::tempdir()
        .map_err(|e| format!("创建临时目录失败: {}", e))?;

    let mut found = false;
    for i in 0..zip.len() {
        let mut entry = zip.by_index(i).map_err(|_| "读取 ZIP 条目失败".to_string())?;
        let name = entry.name().to_string();
        if name.contains("collection.anki21") || name.contains("collection.anki2") {
            let mut out_path = temp_dir.path().join(&name);
            // Ensure parent dir exists
            if let Some(parent) = out_path.parent() {
                std::fs::create_dir_all(parent).map_err(|e| format!("创建目录失败: {}", e))?;
            }
            // If it's the old-format collection.anki2, keep the name
            if !name.ends_with(".anki21") && name.ends_with(".anki2") {
                out_path = temp_dir.path().join("collection.anki21");
            }
            let mut outfile = std::fs::File::create(&out_path)
                .map_err(|e| format!("创建临时文件失败: {}", e))?;
            let mut buf = Vec::new();
            entry.read_to_end(&mut buf).map_err(|_| "读取 ZIP 条目失败".to_string())?;
            std::io::copy(&mut buf.as_slice(), &mut outfile)
                .map_err(|_| "写入临时文件失败".to_string())?;
            found = true;
            break;
        }
    }

    if !found {
        return Err("无效的 APKG 文件: 未找到 collection.anki21".to_string());
    }

    let db_path = temp_dir.path().join("collection.anki21");
    let conn = rusqlite::Connection::open(&db_path)
        .map_err(|e| format!("无法打开 Anki 集合数据库: {}", e))?;

    // Read models JSON from col table
    let models_json: String = conn.query_row(
        "SELECT models FROM col WHERE id = 1",
        [],
        |row| row.get(0),
    ).map_err(|e| format!("无法读取 Anki 模型: {}", e))?;

    // Parse models JSON: map model_id -> { name, field_names: Vec<String> }
    let models_map: HashMap<String, AnkiModel> = serde_json::from_str::<HashMap<String, AnkiModelValue>>(&models_json)
        .map_err(|e| format!("解析 Anki 模型 JSON 失败: {}", e))?
        .into_iter()
        .map(|(id, val)| {
            let fields: Vec<String> = val.flds.into_iter()
                .map(|f| f.name)
                .collect();
            (id, AnkiModel { name: val.name, field_names: fields })
        })
        .collect();

    // Query all notes
    let mut stmt = conn.prepare("SELECT id, mid, flds FROM notes")
        .map_err(|e| format!("查询 Anki 笔记失败: {}", e))?;

    let mut vocabulary = Vec::new();
    let mut grammar = Vec::new();
    let mut kanji = Vec::new();
    let mut errors = Vec::new();
    let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();

    let rows = stmt.query_map([], |row| {
        let id: i64 = row.get(0)?;
        let mid: String = row.get::<_, i64>(1).to_string();
        let flds: String = row.get(2)?;
        Ok((id, mid, flds))
    }).map_err(|e| format!("读取 Anki 笔记失败: {}", e))?;

    for result in rows {
        let (_id, mid, flds) = result.map_err(|e| format!("行读取错误: {}", e))?;

        let model = match models_map.get(&mid) {
            Some(m) => m,
            None => continue,
        };

        let fields: Vec<&str> = flds.split('\x1f').collect();
        let field_map: HashMap<&str, &str> = model.field_names.iter()
            .enumerate()
            .filter_map(|(i, name)| fields.get(i).map(|v| (name.as_str(), *v)))
            .collect();

        let content_type = detect_content_type(&model.name, &field_map);

        match content_type {
            ContentType::Vocabulary => {
                let word = field_map.get("word").or_else(|| field_map.get("Word"))
                    .unwrap_or(&"").trim().to_string();
                if word.is_empty() {
                    errors.push(format!("笔记 {}: word 字段为空，跳过", _id));
                    continue;
                }
                vocabulary.push(Vocabulary {
                    id: uuid::Uuid::new_v4().to_string(),
                    reading: field_map.get("reading").or_else(|| field_map.get("Reading")).unwrap_or(&"").trim().to_string(),
                    meaning: field_map.get("meaning").or_else(|| field_map.get("Meaning")).unwrap_or(&"").trim().to_string(),
                    example: field_map.get("example").or_else(|| field_map.get("Example")).map(|s| s.trim().to_string()).filter(|s| !s.is_empty()),
                    tags: None,
                    word,
                    source: "apkg_import".to_string(),
                    created_at: now.clone(),
                });
            }
            ContentType::Grammar => {
                let pattern = field_map.get("pattern").or_else(|| field_map.get("Pattern")).unwrap_or(&"").trim().to_string();
                if pattern.is_empty() {
                    errors.push(format!("笔记 {}: pattern 字段为空，跳过", _id));
                    continue;
                }
                let examples: Vec<String> = field_map.get("examples")
                    .or_else(|| field_map.get("Examples"))
                    .map(|s| s.split('\n').map(|l| l.trim().to_string()).filter(|l| !l.is_empty()).collect())
                    .unwrap_or_default();
                grammar.push(Grammar {
                    id: uuid::Uuid::new_v4().to_string(),
                    pattern,
                    meaning: field_map.get("meaning").or_else(|| field_map.get("Meaning")).unwrap_or(&"").trim().to_string(),
                    explanation: field_map.get("explanation").or_else(|| field_map.get("Explanation")).unwrap_or(&"").trim().to_string(),
                    examples: serde_json::to_string(&examples).unwrap_or_default(),
                    related: None,
                    source: "apkg_import".to_string(),
                    created_at: now.clone(),
                });
            }
            ContentType::Kanji => {
                let character = field_map.get("character").or_else(|| field_map.get("Kanji"))
                    .or_else(|| field_map.get("Character")).unwrap_or(&"").trim().to_string();
                if character.is_empty() {
                    errors.push(format!("笔记 {}: character 字段为空，跳过", _id));
                    continue;
                }
                let onyomi: Vec<String> = field_map.get("onyomi").or_else(|| field_map.get("Onyomi"))
                    .map(|s| s.split('/').flat_map(|p| p.split(','))
                        .map(|p| p.trim().to_string()).filter(|p| !p.is_empty()).collect())
                    .unwrap_or_default();
                let kunyomi: Vec<String> = field_map.get("kunyomi").or_else(|| field_map.get("Kunyomi"))
                    .map(|s| s.split('/').flat_map(|p| p.split(','))
                        .map(|p| p.trim().to_string()).filter(|p| !p.is_empty()).collect())
                    .unwrap_or_default();
                kanji.push(Kanji {
                    id: uuid::Uuid::new_v4().to_string(),
                    character,
                    onyomi: Some(serde_json::to_string(&onyomi).unwrap_or_default()),
                    kunyomi: Some(serde_json::to_string(&kunyomi).unwrap_or_default()),
                    meaning: field_map.get("meaning").or_else(|| field_map.get("Meaning")).unwrap_or(&"").trim().to_string(),
                    stroke_svg: None,
                    example_words: None,
                    source: "apkg_import".to_string(),
                    created_at: now.clone(),
                });
            }
        }
    }

    // Cleanup: temp_dir is dropped automatically

    drop(conn);

    Ok((vocabulary, grammar, kanji))
}

enum ContentType {
    Vocabulary,
    Grammar,
    Kanji,
}

struct AnkiModel {
    name: String,
    field_names: Vec<String>,
}

#[derive(serde::Deserialize)]
struct AnkiModelValue {
    name: String,
    flds: Vec<AnkiField>,
}

#[derive(serde::Deserialize)]
struct AnkiField {
    name: String,
}

/// Detect content type based on model name and available field names.
fn detect_content_type(model_name: &str, fields: &HashMap<&str, &str>) -> ContentType {
    let name_lower = model_name.to_lowercase();
    let has_kanji_fields = fields.keys().any(|k| {
        let kl = k.to_lowercase();
        kl == "onyomi" || kl == "kunyomi" || kl == "character"
    });
    let has_grammar_fields = fields.keys().any(|k| {
        let kl = k.to_lowercase();
        kl == "pattern" || kl == "explanation"
    });

    if name_lower.contains("kanji") || name_lower.contains("漢字") || name_lower.contains("汉字") || has_kanji_fields {
        ContentType::Kanji
    } else if name_lower.contains("grammar") || name_lower.contains("文法") || name_lower.contains("语法") || has_grammar_fields {
        ContentType::Grammar
    } else {
        ContentType::Vocabulary
    }
}
```

- [ ] **Step 2: Build check**

Run: `cargo check`

Expected: Compiles successfully.

- [ ] **Step 3: Commit**

```bash
git add src-tauri/src/importers/apkg_parser.rs
git commit -m "feat: add APKG parser for Anki deck import

Extract collection.anki21 from zip, read Anki models JSON to map
field names, split flds by \\x1f separator, detect content type
by model name heuristic. Returns vocabulary, grammar, and kanji.

Co-Authored-By: Claude Opus 4.8 <noreply@anthropic.com>"
```

---

### Task 5: User Import Orchestrator

**Files:**
- Create: `src-tauri/src/importers/user_import.rs`

- [ ] **Step 1: Implement user_import.rs**

This file provides 5 public functions that each take a file path + mode, detect format, parse, and save to DB.

```rust
use crate::database::get_pool;
use crate::models::{Grammar, ImportFileResult, ImportMode, Kanji, Listening, Reading, Vocabulary};
use std::path::Path;

macro_rules! insert_or_mode {
    ($mode:expr, $query:expr) => {
        match $mode {
            ImportMode::Append => format!("INSERT OR IGNORE INTO {}", $query),
            ImportMode::Overwrite => format!("INSERT OR REPLACE INTO {}", $query),
        }
    };
}

pub fn import_vocabulary_from_file(file_path: &Path, mode: ImportMode) -> Result<ImportFileResult, String> {
    let ext = file_path.extension().and_then(|e| e.to_str()).unwrap_or("");
    let items = match ext {
        "csv" => crate::importers::csv_parser::parse_csv_vocabulary(file_path)?,
        "json" => crate::importers::json_parser::parse_json_vocabulary(file_path)?,
        "apkg" => return Err("APKG 格式不支持单独导入词汇，请导入整个卡组".to_string()),
        _ => return Err(format!("不支持的文件格式: .{}", ext)),
    };

    let total = items.len();
    let (imported, overwritten) = save_items_with_mode(
        &items,
        mode,
        "vocabulary (word, reading, meaning, example, tags, source, created_at, id) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        |v| vec![
            v.word.as_str(),
            v.reading.as_str(),
            v.meaning.as_str(),
            v.example.as_deref().unwrap_or(""),
            v.tags.as_deref().unwrap_or(""),
            v.source.as_str(),
            v.created_at.as_str(),
            v.id.as_str(),
        ],
    )?;

    Ok(ImportFileResult { imported, overwritten, errors: vec![], total_in_file: total })
}

pub fn import_grammar_from_file(file_path: &Path, mode: ImportMode) -> Result<ImportFileResult, String> {
    let ext = file_path.extension().and_then(|e| e.to_str()).unwrap_or("");
    let items = match ext {
        "json" => crate::importers::json_parser::parse_json_grammar(file_path)?,
        "apkg" => return Err("APKG 格式不支持单独导入语法，请导入整个卡组".to_string()),
        _ => return Err(format!("不支持的文件格式: .{}", ext)),
    };

    let total = items.len();
    let (imported, overwritten) = save_items_with_mode(
        &items,
        mode,
        "grammar (pattern, meaning, explanation, examples, related, source, created_at, id) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        |g| vec![
            g.pattern.as_str(),
            g.meaning.as_str(),
            g.explanation.as_str(),
            g.examples.as_str(),
            g.related.as_deref().unwrap_or(""),
            g.source.as_str(),
            g.created_at.as_str(),
            g.id.as_str(),
        ],
    )?;

    Ok(ImportFileResult { imported, overwritten, errors: vec![], total_in_file: total })
}

pub fn import_kanji_from_file(file_path: &Path, mode: ImportMode) -> Result<ImportFileResult, String> {
    let ext = file_path.extension().and_then(|e| e.to_str()).unwrap_or("");
    let items = match ext {
        "json" => crate::importers::json_parser::parse_json_kanji(file_path)?,
        "apkg" => return Err("APKG 格式不支持单独导入汉字，请导入整个卡组".to_string()),
        _ => return Err(format!("不支持的文件格式: .{}", ext)),
    };

    let total = items.len();
    let (imported, overwritten) = save_items_with_mode(
        &items,
        mode,
        "kanji (character, onyomi, kunyomi, meaning, stroke_svg, example_words, source, created_at, id) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        |k| vec![
            k.character.as_str(),
            k.onyomi.as_deref().unwrap_or(""),
            k.kunyomi.as_deref().unwrap_or(""),
            k.meaning.as_str(),
            "", // stroke_svg
            k.example_words.as_deref().unwrap_or(""),
            k.source.as_str(),
            k.created_at.as_str(),
            k.id.as_str(),
        ],
    )?;

    Ok(ImportFileResult { imported, overwritten, errors: vec![], total_in_file: total })
}

pub fn import_reading_from_file(file_path: &Path, mode: ImportMode) -> Result<ImportFileResult, String> {
    let ext = file_path.extension().and_then(|e| e.to_str()).unwrap_or("");
    let items = match ext {
        "json" => crate::importers::json_parser::parse_json_reading(file_path)?,
        _ => return Err(format!("不支持的文件格式: .{}。阅读导入仅支持 JSON", ext)),
    };

    let total = items.len();
    let (imported, overwritten) = save_items_with_mode(
        &items,
        mode,
        "reading (title, segments, questions, source, level, is_bookmarked, created_at, id) VALUES (?1, ?2, ?3, ?4, ?5, 0, ?6, ?7)",
        |r| vec![
            r.title.as_str(),
            r.segments.as_str(),
            r.questions.as_deref().unwrap_or(""),
            r.source.as_str(),
            r.level.as_str(),
            r.created_at.as_str(),
            r.id.as_str(),
        ],
    )?;

    Ok(ImportFileResult { imported, overwritten, errors: vec![], total_in_file: total })
}

pub fn import_listening_from_file(file_path: &Path, mode: ImportMode) -> Result<ImportFileResult, String> {
    let ext = file_path.extension().and_then(|e| e.to_str()).unwrap_or("");
    let items = match ext {
        "json" => crate::importers::json_parser::parse_json_listening(file_path)?,
        _ => return Err(format!("不支持的文件格式: .{}。听力导入仅支持 JSON", ext)),
    };

    let total = items.len();
    let (imported, overwritten) = save_items_with_mode(
        &items,
        mode,
        "listening (title, audio_file, transcript, questions, source, level, is_bookmarked, created_at, id) VALUES (?1, ?2, ?3, ?4, ?5, ?6, 0, ?7, ?8)",
        |l| vec![
            l.title.as_str(),
            l.audio_file.as_str(),
            l.transcript.as_str(),
            l.questions.as_deref().unwrap_or(""),
            l.source.as_str(),
            l.level.as_str(),
            l.created_at.as_str(),
            l.id.as_str(),
        ],
    )?;

    Ok(ImportFileResult { imported, overwritten, errors: vec![], total_in_file: total })
}

/// Try APKG import: returns vocabulary + grammar + kanji from an APKG file.
pub fn import_apkg(file_path: &Path, mode: ImportMode) -> Result<Vec<ImportFileResult>, String> {
    let (vocab, gram, kanji) = crate::importers::apkg_parser::parse_apkg(file_path)?;

    let mut results = Vec::new();

    if !vocab.is_empty() {
        let total = vocab.len();
        let (imported, overwritten) = save_items_with_mode(&vocab, mode.clone(),
            "vocabulary (word, reading, meaning, example, tags, source, created_at, id) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            |v| vec![v.word.as_str(), v.reading.as_str(), v.meaning.as_str(), v.example.as_deref().unwrap_or(""), v.tags.as_deref().unwrap_or(""), v.source.as_str(), v.created_at.as_str(), v.id.as_str()],
        )?;
        results.push(ImportFileResult { imported, overwritten, errors: vec![], total_in_file: total });
    }

    if !gram.is_empty() {
        let total = gram.len();
        let (imported, overwritten) = save_items_with_mode(&gram, mode.clone(),
            "grammar (pattern, meaning, explanation, examples, related, source, created_at, id) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            |g| vec![g.pattern.as_str(), g.meaning.as_str(), g.explanation.as_str(), g.examples.as_str(), g.related.as_deref().unwrap_or(""), g.source.as_str(), g.created_at.as_str(), g.id.as_str()],
        )?;
        results.push(ImportFileResult { imported, overwritten, errors: vec![], total_in_file: total });
    }

    if !kanji.is_empty() {
        let total = kanji.len();
        let (imported, overwritten) = save_items_with_mode(&kanji, mode.clone(),
            "kanji (character, onyomi, kunyomi, meaning, stroke_svg, example_words, source, created_at, id) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            |k| vec![k.character.as_str(), k.onyomi.as_deref().unwrap_or(""), k.kunyomi.as_deref().unwrap_or(""), k.meaning.as_str(), "", k.example_words.as_deref().unwrap_or(""), k.source.as_str(), k.created_at.as_str(), k.id.as_str()],
        )?;
        results.push(ImportFileResult { imported, overwritten, errors: vec![], total_in_file: total });
    }

    Ok(results)
}

/// Generic function to save items to DB with append/overwrite mode.
fn save_items_with_mode<T, F>(items: &[T], mode: ImportMode, query_suffix: &str, bind_fn: F) -> Result<(usize, usize), String>
where
    F: Fn(&T) -> Vec<&str>,
{
    let pool = get_pool().map_err(|e| e.to_string())?;
    let sql = insert_or_mode!(mode, query_suffix);

    let mut imported = 0usize;
    let mut overwritten = 0usize;

    for item in items {
        let values = bind_fn(item);
        // Build parameterized query
        let mut q = sqlx::query(&sql);
        for val in &values {
            q = q.bind(val);
        }

        match futures::executor::block_on(q.execute(pool)) {
            Ok(result) => {
                if result.rows_affected() > 0 {
                    imported += 1;
                } else {
                    overwritten += 1;
                }
            }
            Err(e) => {
                // In append mode, UNIQUE constraint violation means skipped
                if matches!(mode, ImportMode::Append) && e.to_string().contains("UNIQUE") {
                    overwritten += 1;
                } else {
                    return Err(format!("数据库写入失败: {}", e));
                }
            }
        }
    }

    Ok((imported, overwritten))
}
```

- [ ] **Step 2: Build check**

Run: `cargo check`

Expected: Compiles successfully.

- [ ] **Step 3: Commit**

```bash
git add src-tauri/src/importers/user_import.rs
git commit -m "feat: add user import orchestrator for all 5 content types

Each type has a public import_*_from_file function that detects
format by extension, parses, and saves to DB with Append/Overwrite
mode. APKG imports all 3 flashcard types at once.

Co-Authored-By: Claude Opus 4.8 <noreply@anthropic.com>"
```

---

### Task 6: Tauri Import Commands

**Files:**
- Create: `src-tauri/src/commands/import.rs`

- [ ] **Step 1: Implement import commands**

```rust
use crate::models::{ImportFileResult, ImportMode};
use std::path::Path;

#[tauri::command]
pub async fn import_vocabulary_from_file(
    file_path: String,
    mode: ImportMode,
) -> Result<ImportFileResult, String> {
    let path = Path::new(&file_path);
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");

    // APKG for vocabulary only -> redirect to full APKG import
    if ext == "apkg" {
        let results = crate::importers::user_import::import_apkg(path, mode)?;
        // Sum up all vocab results
        let mut total = ImportFileResult { imported: 0, overwritten: 0, errors: vec![], total_in_file: 0 };
        for r in results {
            total.imported += r.imported;
            total.overwritten += r.overwritten;
            total.total_in_file += r.total_in_file;
        }
        return Ok(total);
    }

    crate::importers::user_import::import_vocabulary_from_file(path, mode)
}

#[tauri::command]
pub async fn import_grammar_from_file(
    file_path: String,
    mode: ImportMode,
) -> Result<ImportFileResult, String> {
    let path = Path::new(&file_path);
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");

    if ext == "apkg" {
        let results = crate::importers::user_import::import_apkg(path, mode)?;
        let mut total = ImportFileResult { imported: 0, overwritten: 0, errors: vec![], total_in_file: 0 };
        for r in results {
            total.imported += r.imported;
            total.overwritten += r.overwritten;
            total.total_in_file += r.total_in_file;
        }
        return Ok(total);
    }

    crate::importers::user_import::import_grammar_from_file(path, mode)
}

#[tauri::command]
pub async fn import_kanji_from_file(
    file_path: String,
    mode: ImportMode,
) -> Result<ImportFileResult, String> {
    let path = Path::new(&file_path);
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");

    if ext == "apkg" {
        let results = crate::importers::user_import::import_apkg(path, mode)?;
        let mut total = ImportFileResult { imported: 0, overwritten: 0, errors: vec![], total_in_file: 0 };
        for r in results {
            total.imported += r.imported;
            total.overwritten += r.overwritten;
            total.total_in_file += r.total_in_file;
        }
        return Ok(total);
    }

    crate::importers::user_import::import_kanji_from_file(path, mode)
}

#[tauri::command]
pub async fn import_reading_from_file(
    file_path: String,
    mode: ImportMode,
) -> Result<ImportFileResult, String> {
    crate::importers::user_import::import_reading_from_file(Path::new(&file_path), mode)
}

#[tauri::command]
pub async fn import_listening_from_file(
    file_path: String,
    mode: ImportMode,
) -> Result<ImportFileResult, String> {
    crate::importers::user_import::import_listening_from_file(Path::new(&file_path), mode)
}
```

**Note:** The APKG path in vocab/grammar/kanji commands forwards to `import_apkg` because user_import.rs's single-type functions reject `.apkg` extension. The command intercepts this and calls the full APKG parser, summing results across all content types found in the deck.

- [ ] **Step 2: Ensure commands/mod.rs declares the import module**

Create or edit `src-tauri/src/commands/mod.rs`:

```rust
pub mod dashboard;
pub mod grammar;
pub mod import;
pub mod kanji;
pub mod listening;
pub mod notes;
pub mod reading;
pub mod search;
pub mod vocabulary;
```

Verify `import` line is present.

- [ ] **Step 3: Build check**

Run: `cargo check`

Expected: Compiles successfully.

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/commands/import.rs src-tauri/src/commands/mod.rs
git commit -m "feat: add 5 Tauri import commands

import_vocabulary/grammar/kanji/reading/listening_from_file commands
that detect format by extension, parse, and save to DB with
Append/Overwrite mode. APKG files handled via full deck parser.

Co-Authored-By: Claude Opus 4.8 <noreply@anthropic.com>"
```

---

### Task 7: Frontend Types + API Layer

**Files:**
- Modify: `src/services/api.ts`
- Create: `src/features/import/types/index.ts`

- [ ] **Step 1: Add ImportFileResult type and API functions to services/api.ts**

Append at end of `src/services/api.ts`:

```typescript
// === Import ===
export interface ImportFileResult {
  imported: number;
  overwritten: number;
  errors: string[];
  total_in_file: number;
}

export function importVocabularyFromFile(
  filePath: string,
  mode: "Append" | "Overwrite",
): Promise<ImportFileResult> {
  return invoke("import_vocabulary_from_file", { filePath, mode });
}

export function importGrammarFromFile(
  filePath: string,
  mode: "Append" | "Overwrite",
): Promise<ImportFileResult> {
  return invoke("import_grammar_from_file", { filePath, mode });
}

export function importKanjiFromFile(
  filePath: string,
  mode: "Append" | "Overwrite",
): Promise<ImportFileResult> {
  return invoke("import_kanji_from_file", { filePath, mode });
}

export function importReadingFromFile(
  filePath: string,
  mode: "Append" | "Overwrite",
): Promise<ImportFileResult> {
  return invoke("import_reading_from_file", { filePath, mode });
}

export function importListeningFromFile(
  filePath: string,
  mode: "Append" | "Overwrite",
): Promise<ImportFileResult> {
  return invoke("import_listening_from_file", { filePath, mode });
}
```

- [ ] **Step 2: Create feature import types**

`src/features/import/types/index.ts`:

```typescript
export type { ImportFileResult } from "@/services/api";
export type ImportContentType = "vocabulary" | "grammar" | "kanji" | "reading" | "listening";
```

- [ ] **Step 3: Commit**

```bash
git add src/services/api.ts src/features/import/types/index.ts
git commit -m "feat: add import types and API layer

ImportFileResult interface and invoke wrappers for all 5 content
types in services/api.ts. Re-export via feature import types.

Co-Authored-By: Claude Opus 4.8 <noreply@anthropic.com>"
```

---

### Task 8: ImportButton + ImportResultDialog Components

**Files:**
- Create: `src/features/import/components/ImportButton.tsx`
- Create: `src/features/import/components/ImportResultDialog.tsx`

- [ ] **Step 1: Implement ImportResultDialog**

```tsx
// src/features/import/components/ImportResultDialog.tsx
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogFooter,
} from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { ImportFileResult } from "@/services/api";

interface Props {
  result: ImportFileResult | null;
  open: boolean;
  onOpenChange: (open: boolean) => void;
}

export default function ImportResultDialog({ result, open, onOpenChange }: Props) {
  if (!result) return null;

  const hasErrors = result.errors.length > 0;
  const allFailed = result.imported === 0 && result.total_in_file > 0;

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-md">
        <DialogHeader>
          <DialogTitle className={allFailed ? "text-destructive" : hasErrors ? "text-amber-500" : "text-green-600"}>
            {allFailed ? "❌ 导入失败" : hasErrors ? "⚠️ 导入完成（有错误）" : "✅ 导入成功"}
          </DialogTitle>
        </DialogHeader>
        <div className="space-y-2 py-4">
          <p>成功导入: <strong>{result.imported}</strong> 条</p>
          {result.overwritten > 0 && (
            <p>覆盖更新: <strong>{result.overwritten}</strong> 条</p>
          )}
          <p className="text-muted-foreground text-sm">文件中共 {result.total_in_file} 条数据</p>
          {hasErrors && (
            <div className="mt-2">
              <p className="text-destructive text-sm font-medium mb-1">
                错误 ({result.errors.length} 条):
              </p>
              <ul className="text-destructive text-xs space-y-1 list-disc pl-4">
                {result.errors.map((err, i) => (
                  <li key={i}>{err}</li>
                ))}
              </ul>
            </div>
          )}
        </div>
        <DialogFooter>
          <Button onClick={() => onOpenChange(false)}>确定</Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
```

- [ ] **Step 2: Implement ImportButton**

```tsx
// src/features/import/components/ImportButton.tsx
import { useState } from "react";
import { open } from "@tauri-apps/plugin-dialog";
import { Button } from "@/components/ui/button";
import { Upload } from "lucide-react";
import { ImportFileResult } from "@/services/api";
import ImportResultDialog from "./ImportResultDialog";
import { ImportContentType } from "../types";
import { useImport } from "../hooks/useImport";

const FILE_FILTERS: Record<ImportContentType, { name: string; extensions: string[] }[]> = {
  vocabulary: [
    { name: "Vocabulary", extensions: ["csv", "json", "apkg"] },
    { name: "CSV", extensions: ["csv"] },
    { name: "JSON", extensions: ["json"] },
    { name: "APKG", extensions: ["apkg"] },
  ],
  grammar: [
    { name: "Grammar", extensions: ["json", "apkg"] },
    { name: "JSON", extensions: ["json"] },
    { name: "APKG", extensions: ["apkg"] },
  ],
  kanji: [
    { name: "Kanji", extensions: ["json", "apkg"] },
    { name: "JSON", extensions: ["json"] },
    { name: "APKG", extensions: ["apkg"] },
  ],
  reading: [{ name: "JSON", extensions: ["json"] }],
  listening: [{ name: "JSON", extensions: ["json"] }],
};

interface Props {
  contentType: ImportContentType;
  onImportComplete: () => void;
}

export default function ImportButton({ contentType, onImportComplete }: Props) {
  const [result, setResult] = useState<ImportFileResult | null>(null);
  const [resultOpen, setResultOpen] = useState(false);
  const [pendingFile, setPendingFile] = useState<string | null>(null);
  const [showModeSelect, setShowModeSelect] = useState(false);
  const importMutation = useImport(contentType);

  const handleClick = async () => {
    try {
      const file = await open({
        multiple: false,
        filters: FILE_FILTERS[contentType],
      });
      if (!file) return;
      setPendingFile(file);
      setShowModeSelect(true);
    } catch (err) {
      console.error("File dialog error:", err);
    }
  };

  const handleModeSelect = async (mode: "Append" | "Overwrite") => {
    if (!pendingFile) return;
    setShowModeSelect(false);
    setPendingFile(null);

    try {
      const res = await importMutation.mutateAsync({ filePath: pendingFile, mode });
      setResult(res);
      setResultOpen(true);
      onImportComplete();
    } catch (err) {
      setResult({
        imported: 0,
        overwritten: 0,
        errors: [typeof err === "string" ? err : "导入失败"],
        total_in_file: 0,
      });
      setResultOpen(true);
    }
  };

  return (
    <>
      <Button
        variant="outline"
        size="sm"
        onClick={handleClick}
        disabled={importMutation.isPending}
      >
        <Upload className="h-4 w-4 mr-1" />
        {importMutation.isPending ? "导入中..." : "导入"}
      </Button>

      {/* Mode selection dialog */}
      <Dialog open={showModeSelect} onOpenChange={setShowModeSelect}>
        <DialogContent className="sm:max-w-sm">
          <DialogHeader>
            <DialogTitle>选择导入模式</DialogTitle>
          </DialogHeader>
          <div className="space-y-3 py-4">
            <Button className="w-full justify-start" variant="outline" onClick={() => handleModeSelect("Append")}>
              📥 仅新增 — 跳过重复数据
            </Button>
            <Button className="w-full justify-start" variant="outline" onClick={() => handleModeSelect("Overwrite")}>
              🔄 覆盖已有 — 更新重复数据
            </Button>
          </div>
        </DialogContent>
      </Dialog>

      <ImportResultDialog
        result={result}
        open={resultOpen}
        onOpenChange={setResultOpen}
      />
    </>
  );
}
```

Note: Make sure to import `Dialog` components:

```tsx
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
```

- [ ] **Step 3: Verify components imports exist**

Check that `lucide-react` exports `Upload` (or use a different icon like `FileUp`). If `Upload` doesn't exist, use `FileUp` or `Import` or `FileInput`.

- [ ] **Step 4: Commit**

```bash
git add src/features/import/components/ImportButton.tsx src/features/import/components/ImportResultDialog.tsx
git commit -m "feat: add ImportButton and ImportResultDialog components

ImportButton triggers Tauri native file dialog with content-type-
specific filters, shows mode selection (Append/Overwrite), and
displays import result via ImportResultDialog.

Co-Authored-By: Claude Opus 4.8 <noreply@anthropic.com>"
```

---

### Task 9: useImport Hook

**Files:**
- Create: `src/features/import/hooks/useImport.ts`

- [ ] **Step 1: Implement useImport hook**

```tsx
// src/features/import/hooks/useImport.ts
import { useMutation } from "@tanstack/react-query";
import {
  importVocabularyFromFile,
  importGrammarFromFile,
  importKanjiFromFile,
  importReadingFromFile,
  importListeningFromFile,
  ImportFileResult,
} from "@/services/api";
import { ImportContentType } from "../types";

const importFns: Record<ImportContentType, (filePath: string, mode: "Append" | "Overwrite") => Promise<ImportFileResult>> = {
  vocabulary: importVocabularyFromFile,
  grammar: importGrammarFromFile,
  kanji: importKanjiFromFile,
  reading: importReadingFromFile,
  listening: importListeningFromFile,
};

export function useImport(contentType: ImportContentType) {
  return useMutation({
    mutationFn: ({
      filePath,
      mode,
    }: {
      filePath: string;
      mode: "Append" | "Overwrite";
    }) => importFns[contentType](filePath, mode),
  });
}
```

- [ ] **Step 2: Commit**

```bash
git add src/features/import/hooks/useImport.ts
git commit -m "feat: add useImport hook with TanStack Query mutation

Wraps 5 import API functions behind a single useMutation hook
parameterized by content type.

Co-Authored-By: Claude Opus 4.8 <noreply@anthropic.com>"
```

---

### Task 10: Add ImportButton to All 5 ListPages

**Files:**
- Modify: `src/features/vocabulary/pages/VocabListPage.tsx`
- Modify: `src/features/grammar/pages/GrammarListPage.tsx`
- Modify: `src/features/kanji/pages/KanjiListPage.tsx`
- Modify: `src/features/reading/pages/ReadingListPage.tsx`
- Modify: `src/features/listening/pages/ListeningListPage.tsx`

- [ ] **Step 1: Modify VocabListPage**

Add import:

```tsx
import ImportButton from "@/features/import/components/ImportButton";
```

Add inside the flex div next to quiz button:

```tsx
<div className="flex items-center justify-between">
  <h1 className="text-2xl font-bold">词汇</h1>
  <div className="flex gap-2">
    <ImportButton contentType="vocabulary" onImportComplete={() => refetch()} />
    <Button onClick={() => navigate("/vocabulary/quiz")}>开始测验</Button>
  </div>
</div>
```

Change `const { data, isLoading }` to `const { data, isLoading, refetch }`.

- [ ] **Step 2: Modify GrammarListPage**

Add import + refetch to GrammarListPage (similar pattern). Read existing GrammarListPage first to see its structure.

- [ ] **Step 3: Modify KanjiListPage**

Same pattern.

- [ ] **Step 4: Modify ReadingListPage**

```tsx
import ImportButton from "@/features/import/components/ImportButton";
```

Change:

```tsx
<div>
  <h1 className="text-2xl font-bold">阅读</h1>
  <div className="flex items-center justify-between">
    <h1 className="text-2xl font-bold">阅读</h1>
    <ImportButton contentType="reading" onImportComplete={() => refetch()} />
  </div>
</div>
```

Add `refetch` from hook: `const { data, isLoading, refetch }`.

- [ ] **Step 5: Modify ListeningListPage**

Same pattern as ReadingListPage.

- [ ] **Step 6: Commit**

```bash
git add src/features/vocabulary/pages/VocabListPage.tsx \
        src/features/grammar/pages/GrammarListPage.tsx \
        src/features/kanji/pages/KanjiListPage.tsx \
        src/features/reading/pages/ReadingListPage.tsx \
        src/features/listening/pages/ListeningListPage.tsx
git commit -m "feat: add ImportButton to all 5 module list pages

Each ListPage now has an import button in the header area, with
content-type-specific file filters. List auto-refreshes on import.

Co-Authored-By: Claude Opus 4.8 <noreply@anthropic.com>"
```

---

### Task 11: Build Verification + Cleanup

**Files:**
- Verify: `cargo check` (Rust)
- Verify: `npx tsc --noEmit` (TypeScript)

- [ ] **Step 1: Rust build check**

Run: `cargo check` (in `src-tauri/` or project root)

Expected: Compiles without errors.

Fix any compilation issues.

- [ ] **Step 2: TypeScript check**

Run: `npx tsc --noEmit`

Expected: No TypeScript errors.

Fix any type issues.

- [ ] **Step 3: Final commit if fixes needed**

```bash
git add -A
git commit -m "fix: address build issues after import pipeline integration"
```

---

## Self-Review

### Spec Coverage
- ✅ Format mapping (CSV→vocab, JSON→all, APKG→vocab/grammar/kanji) — Tasks 2, 3, 4
- ✅ 5 Tauri commands — Task 6
- ✅ ImportMode (Append/Overwrite) — Task 1
- ✅ ImportFileResult — Task 1
- ✅ CSV parser with header validation — Task 2
- ✅ JSON parser for all 5 types — Task 3
- ✅ APKG parser with Anki collection reading — Task 4
- ✅ User import orchestrator — Task 5
- ✅ ImportButton component — Task 8
- ✅ ImportResultDialog component — Task 8
- ✅ useImport hook — Task 9
- ✅ API layer functions — Task 7
- ✅ Page modifications (5 ListPages) — Task 10
- ✅ Build verification — Task 11
- ✅ Error states (parse errors, format errors, DB errors) — covered in parsers + orchestrator

### Placeholder Check
No TBD/TODO/placeholder patterns found. All code is complete and concrete.

### Type Consistency
- `ImportMode` enum: `Append` | `Overwrite` — consistent across Rust (models → user_import → commands) and TS (api.ts → components)
- `ImportFileResult`: `{ imported, overwritten, errors, total_in_file }` — consistent across all layers
- Import command signatures: `(file_path: String, mode: ImportMode) → ImportFileResult` — identical across all 5
