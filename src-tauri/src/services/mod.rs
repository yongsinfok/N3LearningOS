use crate::database::get_pool;
use crate::models::{Flashcard, ReviewHistory};

pub async fn get_or_create_flashcard(
    content_id: &str,
    content_type: &str,
) -> Result<Flashcard, sqlx::Error> {
    let pool = get_pool().map_err(|e| sqlx::Error::Protocol(e.to_string()))?;

    let existing = sqlx::query_as::<_, Flashcard>(
        "SELECT * FROM flashcard WHERE content_id = ?1 AND content_type = ?2",
    )
    .bind(content_id)
    .bind(content_type)
    .fetch_optional(pool)
    .await?;

    if let Some(card) = existing {
        return Ok(card);
    }

    let id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S").to_string();

    sqlx::query(
        "INSERT INTO flashcard (id, content_id, content_type, due_date, state, created_at) \
         VALUES (?1, ?2, ?3, ?4, 0, ?5)",
    )
    .bind(&id)
    .bind(content_id)
    .bind(content_type)
    .bind(&now)
    .bind(&now)
    .execute(pool)
    .await?;

    Ok(Flashcard {
        id,
        content_id: content_id.to_string(),
        content_type: content_type.to_string(),
        due_date: now.clone(),
        stability: 0.0,
        difficulty: 0.0,
        elapsed_days: 0,
        scheduled_days: 0,
        reps: 0,
        lapses: 0,
        state: 0,
        last_review: None,
        created_at: now,
    })
}

pub async fn review_card(
    card: &mut Flashcard,
    rating: i32,
    duration_secs: i32,
) -> Result<ReviewHistory, sqlx::Error> {
    let pool = get_pool().map_err(|e| sqlx::Error::Protocol(e.to_string()))?;

    let fsrs_card = crate::fsrs::Card {
        stability: card.stability,
        difficulty: card.difficulty,
        elapsed_days: card.elapsed_days,
        scheduled_days: card.scheduled_days,
        reps: card.reps,
        lapses: card.lapses,
        state: match card.state {
            0 => crate::fsrs::CardState::New,
            1 => crate::fsrs::CardState::Learning,
            2 => crate::fsrs::CardState::Review,
            3 => crate::fsrs::CardState::Relearning,
            _ => crate::fsrs::CardState::New,
        },
    };

    let fsrs_rating = crate::fsrs::Rating::from_i32(rating).unwrap_or(crate::fsrs::Rating::Good);
    let params = crate::fsrs::params::load_params(
        &std::path::PathBuf::from("fsrs_params.json"),
    );

    let result = crate::fsrs::fsrs_review(&fsrs_card, fsrs_rating, &params);

    let now = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S").to_string();
    let due = (chrono::Utc::now()
        + chrono::Duration::days(result.card.scheduled_days as i64))
    .format("%Y-%m-%dT%H:%M:%S")
    .to_string();

    sqlx::query(
        "UPDATE flashcard SET stability = ?1, difficulty = ?2, elapsed_days = ?3, \
         scheduled_days = ?4, reps = ?5, lapses = ?6, state = ?7, due_date = ?8, last_review = ?9 \
         WHERE id = ?10",
    )
    .bind(result.card.stability)
    .bind(result.card.difficulty)
    .bind(result.card.elapsed_days)
    .bind(result.card.scheduled_days)
    .bind(result.card.reps)
    .bind(result.card.lapses)
    .bind(result.card.state as i32)
    .bind(&due)
    .bind(&now)
    .bind(&card.id)
    .execute(pool)
    .await?;

    let history_id = uuid::Uuid::new_v4().to_string();
    sqlx::query(
        "INSERT INTO review_history (id, flashcard_id, review_date, rating, elapsed_secs) \
         VALUES (?1, ?2, ?3, ?4, ?5)",
    )
    .bind(&history_id)
    .bind(&card.id)
    .bind(&now)
    .bind(rating)
    .bind(duration_secs)
    .execute(pool)
    .await?;

    update_daily_stats(rating).await?;

    card.stability = result.card.stability;
    card.difficulty = result.card.difficulty;
    card.elapsed_days = result.card.elapsed_days;
    card.scheduled_days = result.card.scheduled_days;
    card.reps = result.card.reps;
    card.lapses = result.card.lapses;
    card.state = result.card.state as i32;
    card.due_date = due;
    card.last_review = Some(now.clone());

    Ok(ReviewHistory {
        id: history_id,
        flashcard_id: card.id.clone(),
        review_date: now.clone(),
        rating,
        elapsed_secs: Some(duration_secs),
        created_at: now,
    })
}

async fn update_daily_stats(rating: i32) -> Result<(), sqlx::Error> {
    let pool = get_pool().map_err(|e| sqlx::Error::Protocol(e.to_string()))?;
    let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
    let is_correct = if rating >= 2 { 1 } else { 0 };

    sqlx::query(
        "INSERT INTO daily_stats (date, cards_reviewed, correct_count) \
         VALUES (?1, 1, ?2) \
         ON CONFLICT(date) DO UPDATE SET \
         cards_reviewed = cards_reviewed + 1, \
         correct_count = correct_count + ?3",
    )
    .bind(&today)
    .bind(is_correct)
    .bind(is_correct)
    .execute(pool)
    .await?;

    update_streak(&today).await?;
    Ok(())
}

async fn update_streak(today: &str) -> Result<(), sqlx::Error> {
    let pool = get_pool().map_err(|e| sqlx::Error::Protocol(e.to_string()))?;

    let streak = sqlx::query_as::<_, (i32, i32, String)>(
        "SELECT current_streak, longest_streak, COALESCE(last_study_date, '') \
         FROM streak_stats WHERE id = 1",
    )
    .fetch_optional(pool)
    .await?;

    if let Some((curr_streak, longest, last_date)) = streak {
        if last_date == today {
            return Ok(());
        }
        let yesterday = (chrono::Utc::now() - chrono::Duration::days(1))
            .format("%Y-%m-%d")
            .to_string();

        let new_streak = if last_date.is_empty() || last_date == yesterday {
            curr_streak + 1
        } else if last_date == today {
            curr_streak
        } else {
            1
        };

        let new_longest = longest.max(new_streak);
        sqlx::query(
            "UPDATE streak_stats SET current_streak = ?1, longest_streak = ?2, last_study_date = ?3 WHERE id = 1",
        )
        .bind(new_streak)
        .bind(new_longest)
        .bind(today)
        .execute(pool)
        .await?;
    } else {
        sqlx::query(
            "INSERT INTO streak_stats (id, current_streak, longest_streak, last_study_date) \
             VALUES (1, 1, 1, ?1)",
        )
        .bind(today)
        .execute(pool)
        .await?;
    }
    Ok(())
}
