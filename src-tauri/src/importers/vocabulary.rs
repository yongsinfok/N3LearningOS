use crate::models::Vocabulary;
use std::fs;
use std::path::Path;

pub fn import_vocabulary(content_dir: &Path) -> Result<Vec<Vocabulary>, String> {
    let vocab_path = content_dir.join("vocabulary.json");
    if !vocab_path.exists() {
        return Err("vocabulary.json not found".into());
    }

    let content = fs::read_to_string(&vocab_path)
        .map_err(|e| format!("Cannot read: {}", e))?;

    let items: Vec<VocabInput> = serde_json::from_str(&content)
        .map_err(|e| format!("Cannot parse: {}", e))?;

    let vocab: Vec<Vocabulary> = items
        .into_iter()
        .map(|v| Vocabulary {
            id: v.id.unwrap_or_else(|| uuid::Uuid::new_v4().to_string()),
            word: v.word,
            reading: v.reading,
            meaning: v.meaning,
            example: v.example,
            tags: v.tags.map(|t| serde_json::to_string(&t).unwrap_or_default()),
            source: v.source.unwrap_or_else(|| "bundled".to_string()),
            created_at: chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S").to_string(),
        })
        .collect();

    Ok(vocab)
}

pub fn save_vocabulary_to_db(items: &[Vocabulary]) -> Result<usize, sqlx::Error> {
    let pool = crate::database::get_pool().map_err(|e| sqlx::Error::Protocol(e.to_string()))?;
    futures::executor::block_on(async {
        let mut count = 0;
        for v in items {
            sqlx::query(
                "INSERT OR IGNORE INTO vocabulary (id, word, reading, meaning, example, tags, source, created_at) \
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            )
            .bind(&v.id).bind(&v.word).bind(&v.reading).bind(&v.meaning)
            .bind(&v.example).bind(&v.tags).bind(&v.source).bind(&v.created_at)
            .execute(pool).await?;
            count += 1;
        }
        Ok(count)
    })
}

pub fn create_vocab_flashcards() -> Result<usize, sqlx::Error> {
    let pool = crate::database::get_pool().map_err(|e| sqlx::Error::Protocol(e.to_string()))?;
    futures::executor::block_on(async {
        let items = sqlx::query_as::<_, Vocabulary>(
            "SELECT v.* FROM vocabulary v WHERE NOT EXISTS ( \
             SELECT 1 FROM flashcard f WHERE f.content_id = v.id AND f.content_type = 'vocabulary' \
             )",
        )
        .fetch_all(pool).await?;

        let now = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S").to_string();
        let mut count = 0;
        for v in &items {
            let id = uuid::Uuid::new_v4().to_string();
            sqlx::query(
                "INSERT INTO flashcard (id, content_id, content_type, due_date, state, created_at) \
                 VALUES (?1, ?2, 'vocabulary', ?3, 0, ?4)",
            )
            .bind(&id).bind(&v.id).bind(&now).bind(&now)
            .execute(pool).await?;
            count += 1;
        }
        Ok(count)
    })
}

#[derive(Debug, serde::Deserialize)]
struct VocabInput {
    id: Option<String>,
    word: String,
    reading: String,
    meaning: String,
    example: Option<String>,
    tags: Option<Vec<String>>,
    source: Option<String>,
}
