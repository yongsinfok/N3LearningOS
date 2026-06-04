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

#[derive(Debug, Serialize, Deserialize, Clone, sqlx::FromRow)]
pub struct Grammar {
    pub id: String,
    pub pattern: String,
    pub meaning: String,
    pub explanation: String,
    pub examples: String,
    pub related: Option<String>,
    pub source: String,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, sqlx::FromRow)]
pub struct Kanji {
    pub id: String,
    pub character: String,
    pub onyomi: Option<String>,
    pub kunyomi: Option<String>,
    pub meaning: String,
    pub stroke_svg: Option<String>,
    pub example_words: Option<String>,
    pub source: String,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, sqlx::FromRow)]
pub struct Reading {
    pub id: String,
    pub title: String,
    pub segments: String,
    pub questions: Option<String>,
    pub source: String,
    pub level: String,
    pub is_bookmarked: bool,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Segment {
    pub t: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub f: Option<String>,
    pub k: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Question {
    pub id: String,
    pub question: String,
    pub options: Vec<String>,
    pub correct_index: u8,
    pub explanation: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, sqlx::FromRow)]
pub struct Listening {
    pub id: String,
    pub title: String,
    pub audio_file: String,
    pub transcript: String,
    pub questions: Option<String>,
    pub source: String,
    pub level: String,
    pub is_bookmarked: bool,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TranscriptEntry {
    pub start: f64,
    pub end: f64,
    pub text: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, sqlx::FromRow)]
pub struct Note {
    pub id: String,
    pub content_id: String,
    pub content_type: String,
    pub content: String,
    pub created_at: String,
    pub updated_at: String,
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
    pub grammar_imported: usize,
    pub kanji_imported: usize,
    pub reading_imported: usize,
    pub listening_imported: usize,
    pub flashcards_created: usize,
    pub success: bool,
}
