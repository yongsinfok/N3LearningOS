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

// No flashcards for reading — quiz-based assessment instead
pub fn create_reading_flashcards() -> Result<usize, sqlx::Error> {
    Ok(0)
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
