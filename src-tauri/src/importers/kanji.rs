use crate::models::Kanji;
use std::fs;
use std::path::Path;

pub fn import_kanji(content_dir: &Path) -> Result<Vec<Kanji>, String> {
    let path = content_dir.join("kanji.json");
    if !path.exists() { return Err("kanji.json not found".into()); }

    let content = fs::read_to_string(&path).map_err(|e| format!("Cannot read: {}", e))?;
    let items: Vec<KanjiInput> = serde_json::from_str(&content).map_err(|e| format!("Cannot parse: {}", e))?;

    let now = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S").to_string();
    let kanjis: Vec<Kanji> = items.into_iter().map(|k| Kanji {
        id: k.id.unwrap_or_else(|| uuid::Uuid::new_v4().to_string()),
        character: k.character,
        onyomi: k.onyomi.map(|o| serde_json::to_string(&o).unwrap_or_default()),
        kunyomi: k.kunyomi.map(|k| serde_json::to_string(&k).unwrap_or_default()),
        meaning: k.meaning,
        stroke_svg: None,
        example_words: k.example_words.map(|e| serde_json::to_string(&e).unwrap_or_default()),
        source: k.source.unwrap_or_else(|| "bundled".into()),
        created_at: now.clone(),
    }).collect();

    Ok(kanjis)
}

pub fn save_kanji_to_db(items: &[Kanji]) -> Result<usize, sqlx::Error> {
    let pool = crate::database::get_pool().map_err(|e| sqlx::Error::Protocol(e.to_string()))?;
    futures::executor::block_on(async {
        let mut count = 0;
        for k in items {
            sqlx::query(
                "INSERT OR IGNORE INTO kanji (id, character, onyomi, kunyomi, meaning, example_words, source, created_at) VALUES (?1,?2,?3,?4,?5,?6,?7,?8)",
            )
            .bind(&k.id).bind(&k.character).bind(&k.onyomi).bind(&k.kunyomi)
            .bind(&k.meaning).bind(&k.example_words).bind(&k.source).bind(&k.created_at)
            .execute(pool).await?;
            count += 1;
        }
        Ok(count)
    })
}

pub fn create_kanji_flashcards() -> Result<usize, sqlx::Error> {
    let pool = crate::database::get_pool().map_err(|e| sqlx::Error::Protocol(e.to_string()))?;
    futures::executor::block_on(async {
        let items = sqlx::query_as::<_, Kanji>(
            "SELECT k.* FROM kanji k WHERE NOT EXISTS (SELECT 1 FROM flashcard f WHERE f.content_id = k.id AND f.content_type = 'kanji')",
        ).fetch_all(pool).await?;

        let now = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S").to_string();
        let mut count = 0;
        for k in &items {
            let id = uuid::Uuid::new_v4().to_string();
            sqlx::query("INSERT INTO flashcard (id, content_id, content_type, due_date, state, created_at) VALUES (?1,?2,'kanji',?3,0,?4)")
                .bind(&id).bind(&k.id).bind(&now).bind(&now).execute(pool).await?;
            count += 1;
        }
        Ok(count)
    })
}

#[derive(Debug, serde::Deserialize)]
struct KanjiInput {
    id: Option<String>,
    character: String,
    onyomi: Option<Vec<String>>,
    kunyomi: Option<Vec<String>>,
    meaning: String,
    example_words: Option<Vec<String>>,
    source: Option<String>,
}
