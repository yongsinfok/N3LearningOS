use crate::database::get_pool;
use crate::models::{Listening, PaginatedResult};

#[tauri::command]
pub async fn list_listening(
    page: Option<u32>,
    search: Option<String>,
    page_size: Option<u32>,
) -> Result<PaginatedResult<Listening>, String> {
    let pool = get_pool().map_err(|e| e.to_string())?;
    let page = page.unwrap_or(1);
    let page_size = page_size.unwrap_or(20).min(100);
    let offset = ((page.saturating_sub(1)) * page_size) as i64;

    if let Some(ref term) = search {
        let pattern = format!("%{}%", term);
        let count = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM listening WHERE title LIKE ?1",
        )
        .bind(&pattern)
        .fetch_one(pool)
        .await
        .map_err(|e| e.to_string())?;

        let items = sqlx::query_as::<_, Listening>(
            "SELECT * FROM listening WHERE title LIKE ?1 ORDER BY title LIMIT ?2 OFFSET ?3",
        )
        .bind(&pattern)
        .bind(page_size as i64)
        .bind(offset)
        .fetch_all(pool)
        .await
        .map_err(|e| e.to_string())?;

        return Ok(PaginatedResult { items, total: count, page, page_size });
    }

    let count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM listening")
        .fetch_one(pool).await.map_err(|e| e.to_string())?;
    let items = sqlx::query_as::<_, Listening>(
        "SELECT * FROM listening ORDER BY title LIMIT ?1 OFFSET ?2",
    )
    .bind(page_size as i64).bind(offset)
    .fetch_all(pool).await.map_err(|e| e.to_string())?;

    Ok(PaginatedResult { items, total: count, page, page_size })
}

#[tauri::command]
pub async fn get_listening_detail(id: String) -> Result<Listening, String> {
    let pool = get_pool().map_err(|e| e.to_string())?;
    sqlx::query_as::<_, Listening>("SELECT * FROM listening WHERE id = ?1")
        .bind(&id).fetch_optional(pool).await.map_err(|e| e.to_string())?
        .ok_or_else(|| format!("Listening '{}' not found", id))
}

#[tauri::command]
pub async fn get_audio_data(audio_file: String) -> Result<Vec<u8>, String> {
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    let exe_dir = exe.parent().unwrap_or(&std::path::PathBuf::from(".")).to_path_buf();
    let audio_path = exe_dir.join("content").join("audio").join(&audio_file);

    if !audio_path.exists() {
        return Err(format!("Audio file '{}' not found", audio_file));
    }

    std::fs::read(&audio_path).map_err(|e| format!("Cannot read audio: {}", e))
}
