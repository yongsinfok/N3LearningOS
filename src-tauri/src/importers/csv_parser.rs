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

    let word_idx = headers
        .iter()
        .position(|h| h.trim().to_lowercase() == "word")
        .ok_or_else(|| "CSV 缺少 'word' 列".to_string())?;
    let reading_idx = headers
        .iter()
        .position(|h| h.trim().to_lowercase() == "reading")
        .ok_or_else(|| "CSV 缺少 'reading' 列".to_string())?;
    let meaning_idx = headers
        .iter()
        .position(|h| h.trim().to_lowercase() == "meaning")
        .ok_or_else(|| "CSV 缺少 'meaning' 列".to_string())?;
    let example_idx = headers
        .iter()
        .position(|h| h.trim().to_lowercase() == "example");
    let tags_idx = headers
        .iter()
        .position(|h| h.trim().to_lowercase() == "tags");

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
        let example = example_idx
            .and_then(|i| record.get(i))
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty());
        let tags = tags_idx
            .and_then(|i| record.get(i))
            .map(|s| s.trim().to_string())
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
