use crate::database::get_pool;
use crate::models::{Kanji, PaginatedResult};

#[tauri::command]
pub async fn list_kanji(
    page: Option<u32>,
    search: Option<String>,
    page_size: Option<u32>,
) -> Result<PaginatedResult<Kanji>, String> {
    let pool = get_pool().map_err(|e| e.to_string())?;
    let page = page.unwrap_or(1);
    let page_size = page_size.unwrap_or(20).min(100);
    let offset = ((page.saturating_sub(1)) * page_size) as i64;

    if let Some(ref term) = search {
        let pattern = format!("%{}%", term);
        let count = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM kanji WHERE character LIKE ?1 OR meaning LIKE ?2 OR onyomi LIKE ?3 OR kunyomi LIKE ?4",
        )
        .bind(&pattern).bind(&pattern).bind(&pattern).bind(&pattern)
        .fetch_one(pool).await.map_err(|e| e.to_string())?;

        let items = sqlx::query_as::<_, Kanji>(
            "SELECT * FROM kanji WHERE character LIKE ?1 OR meaning LIKE ?2 OR onyomi LIKE ?3 OR kunyomi LIKE ?4 \
             ORDER BY character LIMIT ?5 OFFSET ?6",
        )
        .bind(&pattern).bind(&pattern).bind(&pattern).bind(&pattern)
        .bind(page_size as i64).bind(offset)
        .fetch_all(pool).await.map_err(|e| e.to_string())?;

        return Ok(PaginatedResult { items, total: count, page, page_size });
    } else {
        let count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM kanji")
            .fetch_one(pool).await.map_err(|e| e.to_string())?;

        let items = sqlx::query_as::<_, Kanji>(
            "SELECT * FROM kanji ORDER BY character LIMIT ?1 OFFSET ?2",
        )
        .bind(page_size as i64).bind(offset)
        .fetch_all(pool).await.map_err(|e| e.to_string())?;

        return Ok(PaginatedResult { items, total: count, page, page_size });
    }
}

#[tauri::command]
pub async fn get_kanji_detail(id: String) -> Result<Kanji, String> {
    let pool = get_pool().map_err(|e| e.to_string())?;
    sqlx::query_as::<_, Kanji>("SELECT * FROM kanji WHERE id = ?1")
        .bind(&id)
        .fetch_optional(pool)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("Kanji '{}' not found", id))
}
