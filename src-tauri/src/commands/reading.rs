use crate::database::get_pool;
use crate::models::{PaginatedResult, Reading, Vocabulary};

#[tauri::command]
pub async fn list_reading(
    page: Option<u32>,
    search: Option<String>,
    page_size: Option<u32>,
) -> Result<PaginatedResult<Reading>, String> {
    let pool = get_pool().map_err(|e| e.to_string())?;
    let page = page.unwrap_or(1);
    let page_size = page_size.unwrap_or(20).min(100);
    let offset = ((page.saturating_sub(1)) * page_size) as i64;

    if let Some(ref term) = search {
        let pattern = format!("%{}%", term);
        let count = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM reading WHERE title LIKE ?1",
        )
        .bind(&pattern)
        .fetch_one(pool).await.map_err(|e| e.to_string())?;

        let items = sqlx::query_as::<_, Reading>(
            "SELECT * FROM reading WHERE title LIKE ?1 ORDER BY title LIMIT ?2 OFFSET ?3",
        )
        .bind(&pattern).bind(page_size as i64).bind(offset)
        .fetch_all(pool).await.map_err(|e| e.to_string())?;

        return Ok(PaginatedResult { items, total: count, page, page_size });
    } else {
        let count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM reading")
            .fetch_one(pool).await.map_err(|e| e.to_string())?;

        let items = sqlx::query_as::<_, Reading>(
            "SELECT * FROM reading ORDER BY title LIMIT ?1 OFFSET ?2",
        )
        .bind(page_size as i64).bind(offset)
        .fetch_all(pool).await.map_err(|e| e.to_string())?;

        return Ok(PaginatedResult { items, total: count, page, page_size });
    }
}

#[tauri::command]
pub async fn get_reading_detail(id: String) -> Result<Reading, String> {
    let pool = get_pool().map_err(|e| e.to_string())?;
    sqlx::query_as::<_, Reading>("SELECT * FROM reading WHERE id = ?1")
        .bind(&id)
        .fetch_optional(pool)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("Reading '{}' not found", id))
}

#[tauri::command]
pub async fn lookup_word(word: String) -> Result<Option<Vocabulary>, String> {
    let pool = get_pool().map_err(|e| e.to_string())?;
    let result = sqlx::query_as::<_, Vocabulary>(
        "SELECT * FROM vocabulary WHERE word = ?1 LIMIT 1",
    )
    .bind(&word)
    .fetch_optional(pool)
    .await
    .map_err(|e| e.to_string())?;
    Ok(result)
}
