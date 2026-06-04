use crate::models::{Listening, TranscriptEntry};
use std::fs;
use std::path::Path;

pub fn import_listening(content_dir: &Path) -> Result<Vec<Listening>, String> {
    let path = content_dir.join("listening.json");
    if !path.exists() {
        return Err("listening.json not found".into());
    }
    let content = fs::read_to_string(&path).map_err(|e| format!("Cannot read: {}", e))?;
    let items: Vec<ListeningInput> =
        serde_json::from_str(&content).map_err(|e| format!("Cannot parse: {}", e))?;
    let now = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S").to_string();
    let items: Vec<Listening> = items
        .into_iter()
        .map(|r| Listening {
            id: r.id.unwrap_or_else(|| uuid::Uuid::new_v4().to_string()),
            title: r.title,
            audio_file: r.audio_file,
            transcript: serde_json::to_string(&r.transcript).unwrap_or_default(),
            questions: r
                .questions
                .map(|q| serde_json::to_string(&q).unwrap_or_default()),
            source: r.source.unwrap_or_else(|| "bundled".into()),
            level: r.level.unwrap_or_else(|| "N3".into()),
            is_bookmarked: false,
            created_at: now.clone(),
        })
        .collect();
    Ok(items)
}

pub fn save_listening_to_db(items: &[Listening]) -> Result<usize, sqlx::Error> {
    let pool =
        crate::database::get_pool().map_err(|e| sqlx::Error::Protocol(e.to_string()))?;
    futures::executor::block_on(async {
        let mut count = 0;
        for r in items {
            sqlx::query(
                "INSERT OR IGNORE INTO listening (id, title, audio_file, transcript, questions, source, level, is_bookmarked, created_at) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9)",
            )
            .bind(&r.id)
            .bind(&r.title)
            .bind(&r.audio_file)
            .bind(&r.transcript)
            .bind(&r.questions)
            .bind(&r.source)
            .bind(&r.level)
            .bind(&r.is_bookmarked)
            .bind(&r.created_at)
            .execute(pool)
            .await?;
            count += 1;
        }
        Ok(count)
    })
}

pub fn create_listening_flashcards() -> Result<usize, sqlx::Error> {
    Ok(0)
}

#[derive(Debug, serde::Deserialize)]
struct ListeningInput {
    id: Option<String>,
    title: String,
    audio_file: String,
    transcript: Vec<TranscriptEntry>,
    questions: Option<Vec<crate::models::Question>>,
    source: Option<String>,
    level: Option<String>,
}
