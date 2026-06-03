use crate::models::Grammar;
use std::fs;
use std::path::Path;

pub fn import_grammar(content_dir: &Path) -> Result<Vec<Grammar>, String> {
    let path = content_dir.join("grammar.json");
    if !path.exists() { return Err("grammar.json not found".into()); }

    let content = fs::read_to_string(&path).map_err(|e| format!("Cannot read: {}", e))?;
    let items: Vec<GrammarInput> = serde_json::from_str(&content).map_err(|e| format!("Cannot parse: {}", e))?;

    let now = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S").to_string();
    let grammars: Vec<Grammar> = items.into_iter().map(|g| Grammar {
        id: g.id.unwrap_or_else(|| uuid::Uuid::new_v4().to_string()),
        pattern: g.pattern,
        meaning: g.meaning,
        explanation: g.explanation,
        examples: serde_json::to_string(&g.examples).unwrap_or_default(),
        related: g.related.map(|r| serde_json::to_string(&r).unwrap_or_default()),
        source: g.source.unwrap_or_else(|| "bundled".into()),
        created_at: now.clone(),
    }).collect();

    Ok(grammars)
}

pub fn save_grammar_to_db(items: &[Grammar]) -> Result<usize, sqlx::Error> {
    let pool = crate::database::get_pool().map_err(|e| sqlx::Error::Protocol(e.to_string()))?;
    futures::executor::block_on(async {
        let mut count = 0;
        for g in items {
            sqlx::query(
                "INSERT OR IGNORE INTO grammar (id, pattern, meaning, explanation, examples, related, source, created_at) VALUES (?1,?2,?3,?4,?5,?6,?7,?8)",
            )
            .bind(&g.id).bind(&g.pattern).bind(&g.meaning).bind(&g.explanation)
            .bind(&g.examples).bind(&g.related).bind(&g.source).bind(&g.created_at)
            .execute(pool).await?;
            count += 1;
        }
        Ok(count)
    })
}

pub fn create_grammar_flashcards() -> Result<usize, sqlx::Error> {
    let pool = crate::database::get_pool().map_err(|e| sqlx::Error::Protocol(e.to_string()))?;
    futures::executor::block_on(async {
        let items = sqlx::query_as::<_, Grammar>(
            "SELECT g.* FROM grammar g WHERE NOT EXISTS (SELECT 1 FROM flashcard f WHERE f.content_id = g.id AND f.content_type = 'grammar')",
        ).fetch_all(pool).await?;

        let now = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S").to_string();
        let mut count = 0;
        for g in &items {
            let id = uuid::Uuid::new_v4().to_string();
            sqlx::query("INSERT INTO flashcard (id, content_id, content_type, due_date, state, created_at) VALUES (?1,?2,'grammar',?3,0,?4)")
                .bind(&id).bind(&g.id).bind(&now).bind(&now).execute(pool).await?;
            count += 1;
        }
        Ok(count)
    })
}

#[derive(Debug, serde::Deserialize)]
struct GrammarInput {
    id: Option<String>,
    pattern: String,
    meaning: String,
    explanation: String,
    examples: Vec<String>,
    related: Option<Vec<String>>,
    source: Option<String>,
}
