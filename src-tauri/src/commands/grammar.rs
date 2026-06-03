use crate::database::get_pool;
use crate::models::{Grammar, PaginatedResult};

#[tauri::command]
pub async fn list_grammar(
    page: Option<u32>,
    search: Option<String>,
    page_size: Option<u32>,
) -> Result<PaginatedResult<Grammar>, String> {
    let pool = get_pool().map_err(|e| e.to_string())?;
    let page = page.unwrap_or(1);
    let page_size = page_size.unwrap_or(20).min(100);
    let offset = ((page.saturating_sub(1)) * page_size) as i64;

    if let Some(ref term) = search {
        let pattern = format!("%{}%", term);
        let count = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM grammar WHERE pattern LIKE ?1 OR meaning LIKE ?2 OR explanation LIKE ?3",
        )
        .bind(&pattern).bind(&pattern).bind(&pattern)
        .fetch_one(pool).await.map_err(|e| e.to_string())?;

        let items = sqlx::query_as::<_, Grammar>(
            "SELECT * FROM grammar WHERE pattern LIKE ?1 OR meaning LIKE ?2 OR explanation LIKE ?3 \
             ORDER BY pattern LIMIT ?4 OFFSET ?5",
        )
        .bind(&pattern).bind(&pattern).bind(&pattern)
        .bind(page_size as i64).bind(offset)
        .fetch_all(pool).await.map_err(|e| e.to_string())?;

        return Ok(PaginatedResult { items, total: count, page, page_size });
    } else {
        let count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM grammar")
            .fetch_one(pool).await.map_err(|e| e.to_string())?;

        let items = sqlx::query_as::<_, Grammar>(
            "SELECT * FROM grammar ORDER BY pattern LIMIT ?1 OFFSET ?2",
        )
        .bind(page_size as i64).bind(offset)
        .fetch_all(pool).await.map_err(|e| e.to_string())?;

        return Ok(PaginatedResult { items, total: count, page, page_size });
    }
}

#[tauri::command]
pub async fn get_grammar_detail(id: String) -> Result<Grammar, String> {
    let pool = get_pool().map_err(|e| e.to_string())?;
    sqlx::query_as::<_, Grammar>("SELECT * FROM grammar WHERE id = ?1")
        .bind(&id)
        .fetch_optional(pool)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("Grammar '{}' not found", id))
}
