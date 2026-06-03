use crate::database::get_pool;
use crate::models::Note;

#[tauri::command]
pub async fn list_notes(
    content_id: Option<String>,
    content_type: Option<String>,
) -> Result<Vec<Note>, String> {
    let pool = get_pool().map_err(|e| e.to_string())?;

    let notes = if let (Some(cid), Some(ctype)) = (&content_id, &content_type) {
        sqlx::query_as::<_, Note>(
            "SELECT * FROM note WHERE content_id = ?1 AND content_type = ?2 ORDER BY updated_at DESC",
        )
        .bind(cid).bind(ctype)
    } else {
        sqlx::query_as::<_, Note>(
            "SELECT * FROM note ORDER BY updated_at DESC LIMIT 50",
        )
    };

    notes.fetch_all(pool).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn save_note(
    content_id: String,
    content_type: String,
    content: String,
) -> Result<String, String> {
    let pool = get_pool().map_err(|e| e.to_string())?;
    let now = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S").to_string();

    let existing = sqlx::query_as::<_, Note>(
        "SELECT * FROM note WHERE content_id = ?1 AND content_type = ?2",
    )
    .bind(&content_id).bind(&content_type)
    .fetch_optional(pool)
    .await
    .map_err(|e| e.to_string())?;

    if let Some(note) = existing {
        sqlx::query("UPDATE note SET content = ?1, updated_at = ?2 WHERE id = ?3")
            .bind(&content).bind(&now).bind(&note.id)
            .execute(pool).await.map_err(|e| e.to_string())?;
        Ok(note.id)
    } else {
        let id = uuid::Uuid::new_v4().to_string();
        sqlx::query("INSERT INTO note (id, content_id, content_type, content, created_at, updated_at) VALUES (?1,?2,?3,?4,?5,?6)")
            .bind(&id).bind(&content_id).bind(&content_type).bind(&content).bind(&now).bind(&now)
            .execute(pool).await.map_err(|e| e.to_string())?;
        Ok(id)
    }
}

#[tauri::command]
pub async fn delete_note(id: String) -> Result<(), String> {
    let pool = get_pool().map_err(|e| e.to_string())?;
    sqlx::query("DELETE FROM note WHERE id = ?1")
        .bind(&id)
        .execute(pool).await.map_err(|e| e.to_string())?;
    Ok(())
}
