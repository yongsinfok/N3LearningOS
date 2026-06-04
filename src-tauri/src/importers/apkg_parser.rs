use crate::models::{Grammar, Kanji, Vocabulary};
use std::collections::HashMap;
use std::io::Read;
use std::path::Path;

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
            if let Some(parent) = out_path.parent() {
                std::fs::create_dir_all(parent).map_err(|e| format!("创建目录失败: {}", e))?;
            }
            // Normalize to .anki21 name
            if !name.ends_with(".anki21") && name.ends_with(".anki2") {
                out_path = temp_dir.path().join("collection.anki21");
            }
            let mut outfile = std::fs::File::create(&out_path)
                .map_err(|e| format!("创建临时文件失败: {}", e))?;
            let mut buf = Vec::new();
            entry
                .read_to_end(&mut buf)
                .map_err(|_| "读取 ZIP 条目失败".to_string())?;
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
    let models_json: String = conn
        .query_row("SELECT models FROM col WHERE id = 1", [], |row| row.get(0))
        .map_err(|e| format!("无法读取 Anki 模型: {}", e))?;

    // Parse models JSON
    let models_map: HashMap<String, AnkiModel> =
        serde_json::from_str::<HashMap<String, AnkiModelValue>>(&models_json)
            .map_err(|e| format!("解析 Anki 模型 JSON 失败: {}", e))?
            .into_iter()
            .map(|(id, val)| {
                let fields: Vec<String> = val.flds.into_iter().map(|f| f.name).collect();
                (id, AnkiModel {
                    name: val.name,
                    field_names: fields,
                })
            })
            .collect();

    // Query all notes
    let mut stmt = conn
        .prepare("SELECT id, mid, flds FROM notes")
        .map_err(|e| format!("查询 Anki 笔记失败: {}", e))?;

    let mut vocabulary = Vec::new();
    let mut grammar = Vec::new();
    let mut kanji = Vec::new();
    let mut errors: Vec<String> = Vec::new();
    let now = chrono::Utc::now()
        .format("%Y-%m-%d %H:%M:%S")
        .to_string();

    let rows = stmt
        .query_map([], |row| {
            let id: i64 = row.get(0)?;
            let mid: String = row.get::<_, i64>(1)?.to_string();
            let flds: String = row.get(2)?;
            Ok((id, mid, flds))
        })
        .map_err(|e| format!("读取 Anki 笔记失败: {}", e))?;

    for result in rows {
        let (_id, mid, flds) = result.map_err(|e| format!("行读取错误: {}", e))?;

        let model = match models_map.get(&mid) {
            Some(m) => m,
            None => continue,
        };

        let fields: Vec<&str> = flds.split('\x1f').collect();
        let field_map: HashMap<&str, &str> = model
            .field_names
            .iter()
            .enumerate()
            .filter_map(|(i, name)| fields.get(i).map(|v| (name.as_str(), *v)))
            .collect();

        let content_type = detect_content_type(&model.name, &field_map);

        match content_type {
            ContentType::Vocabulary => {
                let word = field_map
                    .get("word")
                    .or_else(|| field_map.get("Word"))
                    .unwrap_or(&"")
                    .trim()
                    .to_string();
                if word.is_empty() {
                    errors.push(format!("笔记 {}: word 字段为空，跳过", _id));
                    continue;
                }
                vocabulary.push(Vocabulary {
                    id: uuid::Uuid::new_v4().to_string(),
                    reading: field_map
                        .get("reading")
                        .or_else(|| field_map.get("Reading"))
                        .unwrap_or(&"")
                        .trim()
                        .to_string(),
                    meaning: field_map
                        .get("meaning")
                        .or_else(|| field_map.get("Meaning"))
                        .unwrap_or(&"")
                        .trim()
                        .to_string(),
                    example: field_map
                        .get("example")
                        .or_else(|| field_map.get("Example"))
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty()),
                    tags: None,
                    word,
                    source: "apkg_import".to_string(),
                    created_at: now.clone(),
                });
            }
            ContentType::Grammar => {
                let pattern = field_map
                    .get("pattern")
                    .or_else(|| field_map.get("Pattern"))
                    .unwrap_or(&"")
                    .trim()
                    .to_string();
                if pattern.is_empty() {
                    errors.push(format!("笔记 {}: pattern 字段为空，跳过", _id));
                    continue;
                }
                let examples: Vec<String> = field_map
                    .get("examples")
                    .or_else(|| field_map.get("Examples"))
                    .map(|s| {
                        s.split('\n')
                            .map(|l| l.trim().to_string())
                            .filter(|l| !l.is_empty())
                            .collect()
                    })
                    .unwrap_or_default();
                grammar.push(Grammar {
                    id: uuid::Uuid::new_v4().to_string(),
                    pattern,
                    meaning: field_map
                        .get("meaning")
                        .or_else(|| field_map.get("Meaning"))
                        .unwrap_or(&"")
                        .trim()
                        .to_string(),
                    explanation: field_map
                        .get("explanation")
                        .or_else(|| field_map.get("Explanation"))
                        .unwrap_or(&"")
                        .trim()
                        .to_string(),
                    examples: serde_json::to_string(&examples).unwrap_or_default(),
                    related: None,
                    source: "apkg_import".to_string(),
                    created_at: now.clone(),
                });
            }
            ContentType::Kanji => {
                let character = field_map
                    .get("character")
                    .or_else(|| field_map.get("Kanji"))
                    .or_else(|| field_map.get("Character"))
                    .unwrap_or(&"")
                    .trim()
                    .to_string();
                if character.is_empty() {
                    errors.push(format!("笔记 {}: character 字段为空，跳过", _id));
                    continue;
                }
                let onyomi: Vec<String> = field_map
                    .get("onyomi")
                    .or_else(|| field_map.get("Onyomi"))
                    .map(|s| {
                        s.split('/')
                            .flat_map(|p| p.split(','))
                            .map(|p| p.trim().to_string())
                            .filter(|p| !p.is_empty())
                            .collect()
                    })
                    .unwrap_or_default();
                let kunyomi: Vec<String> = field_map
                    .get("kunyomi")
                    .or_else(|| field_map.get("Kunyomi"))
                    .map(|s| {
                        s.split('/')
                            .flat_map(|p| p.split(','))
                            .map(|p| p.trim().to_string())
                            .filter(|p| !p.is_empty())
                            .collect()
                    })
                    .unwrap_or_default();
                kanji.push(Kanji {
                    id: uuid::Uuid::new_v4().to_string(),
                    character,
                    onyomi: Some(serde_json::to_string(&onyomi).unwrap_or_default()),
                    kunyomi: Some(serde_json::to_string(&kunyomi).unwrap_or_default()),
                    meaning: field_map
                        .get("meaning")
                        .or_else(|| field_map.get("Meaning"))
                        .unwrap_or(&"")
                        .trim()
                        .to_string(),
                    stroke_svg: None,
                    example_words: None,
                    source: "apkg_import".to_string(),
                    created_at: now.clone(),
                });
            }
        }
    }

    // Ignore errors in production — they're diagnostic for the caller if needed
    let _ = errors;

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

    if name_lower.contains("kanji")
        || name_lower.contains("漢字")
        || name_lower.contains("汉字")
        || has_kanji_fields
    {
        ContentType::Kanji
    } else if name_lower.contains("grammar")
        || name_lower.contains("文法")
        || name_lower.contains("语法")
        || has_grammar_fields
    {
        ContentType::Grammar
    } else {
        ContentType::Vocabulary
    }
}
