use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, sqlx::FromRow)]
pub struct Vocabulary {
    pub id: String,
    pub word: String,
    pub reading: String,
    pub meaning: String,
    pub example: Option<String>,
    pub tags: Option<String>,
    pub source: String,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, sqlx::FromRow)]
pub struct Flashcard {
    pub id: String,
    pub content_id: String,
    pub content_type: String,
    pub due_date: String,
    pub stability: f64,
    pub difficulty: f64,
    pub elapsed_days: i32,
    pub scheduled_days: i32,
    pub reps: i32,
    pub lapses: i32,
    pub state: i32,
    pub last_review: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, sqlx::FromRow)]
pub struct ReviewHistory {
    pub id: String,
    pub flashcard_id: String,
    pub review_date: String,
    pub rating: i32,
    pub elapsed_secs: Option<i32>,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PaginatedResult<T> {
    pub items: Vec<T>,
    pub total: i64,
    pub page: u32,
    pub page_size: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DashboardData {
    pub streak: i32,
    pub due_count: i64,
    pub new_available: i64,
    pub module_progress: Vec<ModuleProgress>,
    pub weekly_activity: Vec<DailyActivity>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ModuleProgress {
    pub module: String,
    pub completed: i64,
    pub total: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DailyActivity {
    pub date: String,
    pub cards_reviewed: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ImportResult {
    pub vocabulary_imported: usize,
    pub flashcards_created: usize,
    pub success: bool,
}
