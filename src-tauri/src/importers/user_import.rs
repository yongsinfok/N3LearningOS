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
            "",
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

/// Import APKG file: returns vocabulary + grammar + kanji results.
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
