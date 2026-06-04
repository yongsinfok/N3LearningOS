use crate::database::get_pool;
use crate::models::{DailyActivity, DashboardData, ModuleProgress};

#[tauri::command]
pub async fn get_dashboard() -> Result<DashboardData, String> {
    let pool = get_pool().map_err(|e| e.to_string())?;

    let (current_streak, _longest_streak) = sqlx::query_as::<_, (i32, i32)>(
        "SELECT COALESCE(current_streak, 0), COALESCE(longest_streak, 0) \
         FROM streak_stats WHERE id = 1",
    )
    .fetch_optional(pool)
    .await
    .map_err(|e| e.to_string())?
    .unwrap_or((0, 0));

    let now = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S").to_string();

    let due_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM flashcard WHERE due_date <= ?1",
    )
    .bind(&now)
    .fetch_one(pool)
    .await
    .map_err(|e| e.to_string())?;

    let new_available: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM flashcard WHERE state = 0",
    )
    .fetch_one(pool)
    .await
    .map_err(|e| e.to_string())?;

    let vocab_total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM vocabulary")
        .fetch_one(pool)
        .await
        .map_err(|e| e.to_string())?;

    let vocab_done: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM flashcard WHERE content_type = 'vocabulary' AND state >= 2",
    )
    .fetch_one(pool)
    .await
    .map_err(|e| e.to_string())?;

    let grammar_total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM grammar")
        .fetch_one(pool)
        .await
        .map_err(|e| e.to_string())?;
    let grammar_done: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM flashcard WHERE content_type = 'grammar' AND state >= 2",
    )
    .fetch_one(pool)
    .await
    .map_err(|e| e.to_string())?;

    let kanji_total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM kanji")
        .fetch_one(pool)
        .await
        .map_err(|e| e.to_string())?;
    let kanji_done: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM flashcard WHERE content_type = 'kanji' AND state >= 2",
    )
    .fetch_one(pool)
    .await
    .map_err(|e| e.to_string())?;

    let reading_total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM reading")
        .fetch_one(pool)
        .await
        .map_err(|e| e.to_string())?;
    // Reading has no flashcards — track via bookmarks in future
    let reading_done: i64 = 0;

    let listening_total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM listening")
        .fetch_one(pool)
        .await
        .map_err(|e| e.to_string())?;
    let listening_done: i64 = 0;

    let module_progress = vec![
        ModuleProgress { module: "vocabulary".into(), completed: vocab_done, total: vocab_total },
        ModuleProgress { module: "grammar".into(), completed: grammar_done, total: grammar_total },
        ModuleProgress { module: "kanji".into(), completed: kanji_done, total: kanji_total },
        ModuleProgress { module: "reading".into(), completed: reading_done, total: reading_total },
        ModuleProgress { module: "listening".into(), completed: listening_done, total: listening_total },
    ];

    let week_ago = (chrono::Utc::now() - chrono::Duration::days(6))
        .format("%Y-%m-%d")
        .to_string();

    let weekly = sqlx::query_as::<_, (String, i64)>(
        "SELECT date, cards_reviewed FROM daily_stats WHERE date >= ?1 ORDER BY date",
    )
    .bind(&week_ago)
    .fetch_all(pool)
    .await
    .map_err(|e| e.to_string())?;

    let weekly_activity: Vec<DailyActivity> = weekly
        .into_iter()
        .map(|(date, cards)| DailyActivity { date, cards_reviewed: cards })
        .collect();

    Ok(DashboardData {
        streak: current_streak,
        due_count,
        new_available,
        module_progress,
        weekly_activity,
    })
}
