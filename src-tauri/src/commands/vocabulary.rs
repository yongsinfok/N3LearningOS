use crate::database::get_pool;
use crate::models::{Flashcard, PaginatedResult, Vocabulary};

#[tauri::command]
pub async fn list_vocabulary(
    page: Option<u32>,
    search: Option<String>,
    page_size: Option<u32>,
) -> Result<PaginatedResult<Vocabulary>, String> {
    let pool = get_pool().map_err(|e| e.to_string())?;
    let page = page.unwrap_or(1);
    let page_size = page_size.unwrap_or(20).min(100);
    let offset = ((page.saturating_sub(1)) * page_size) as i64;

    if let Some(ref term) = search {
        let pattern = format!("%{}%", term);
        let count = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM vocabulary WHERE word LIKE ?1 OR reading LIKE ?2 OR meaning LIKE ?3",
        )
        .bind(&pattern).bind(&pattern).bind(&pattern)
        .fetch_one(pool).await.map_err(|e| e.to_string())?;

        let items = sqlx::query_as::<_, Vocabulary>(
            "SELECT * FROM vocabulary WHERE word LIKE ?1 OR reading LIKE ?2 OR meaning LIKE ?3 \
             ORDER BY word LIMIT ?4 OFFSET ?5",
        )
        .bind(&pattern).bind(&pattern).bind(&pattern)
        .bind(page_size as i64).bind(offset)
        .fetch_all(pool).await.map_err(|e| e.to_string())?;

        return Ok(PaginatedResult { items, total: count, page, page_size });
    } else {
        let count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM vocabulary")
            .fetch_one(pool).await.map_err(|e| e.to_string())?;

        let items = sqlx::query_as::<_, Vocabulary>(
            "SELECT * FROM vocabulary ORDER BY word LIMIT ?1 OFFSET ?2",
        )
        .bind(page_size as i64).bind(offset)
        .fetch_all(pool).await.map_err(|e| e.to_string())?;

        return Ok(PaginatedResult { items, total: count, page, page_size });
    }
}

#[tauri::command]
pub async fn get_vocabulary_detail(id: String) -> Result<Vocabulary, String> {
    let pool = get_pool().map_err(|e| e.to_string())?;
    sqlx::query_as::<_, Vocabulary>("SELECT * FROM vocabulary WHERE id = ?1")
        .bind(&id)
        .fetch_optional(pool)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("Vocabulary '{}' not found", id))
}

#[tauri::command]
pub async fn get_due_flashcards(
    content_type: Option<String>,
    limit: Option<i64>,
) -> Result<Vec<(Flashcard, Vocabulary)>, String> {
    let pool = get_pool().map_err(|e| e.to_string())?;
    let now = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S").to_string();
    let limit = limit.unwrap_or(20);

    let flashcards = if let Some(ref ctype) = content_type {
        sqlx::query_as::<_, Flashcard>(
            "SELECT * FROM flashcard WHERE content_type = ?1 AND due_date <= ?2 \
             ORDER BY due_date LIMIT ?3",
        )
        .bind(ctype).bind(&now).bind(limit)
    } else {
        sqlx::query_as::<_, Flashcard>(
            "SELECT * FROM flashcard WHERE due_date <= ?1 ORDER BY due_date LIMIT ?2",
        )
        .bind(&now).bind(limit)
    };

    let flashcards = flashcards.fetch_all(pool).await.map_err(|e| e.to_string())?;

    let mut result = Vec::new();
    for card in flashcards {
        if let Ok(vocab) = sqlx::query_as::<_, Vocabulary>(
            "SELECT * FROM vocabulary WHERE id = ?1",
        )
        .bind(&card.content_id)
        .fetch_one(pool)
        .await
        {
            result.push((card, vocab));
        }
    }
    Ok(result)
}

#[tauri::command]
pub async fn review_flashcard(
    card_id: String,
    rating: i32,
    duration_secs: Option<i32>,
) -> Result<String, String> {
    let pool = get_pool().map_err(|e| e.to_string())?;

    let mut card = sqlx::query_as::<_, Flashcard>(
        "SELECT * FROM flashcard WHERE id = ?1",
    )
    .bind(&card_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| e.to_string())?
    .ok_or_else(|| format!("Flashcard '{}' not found", card_id))?;

    crate::services::review_card(&mut card, rating, duration_secs.unwrap_or(0))
        .await
        .map_err(|e| e.to_string())?;

    Ok("ok".to_string())
}
