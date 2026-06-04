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
