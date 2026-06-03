use crate::database::get_pool;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct SearchResult {
    pub id: String,
    pub content_type: String,
    pub title: String,
    pub subtitle: String,
    pub match_field: String,
}

#[tauri::command]
pub async fn global_search(query: String) -> Result<Vec<SearchResult>, String> {
    let pool = get_pool().map_err(|e| e.to_string())?;
    let pattern = format!("%{}%", query);
    let mut results = Vec::new();

    let vocab_matches = sqlx::query_as::<_, (String, String, String, String)>(
        "SELECT id, word, reading, meaning FROM vocabulary WHERE word LIKE ?1 OR reading LIKE ?2 OR meaning LIKE ?3 LIMIT 10",
    )
    .bind(&pattern).bind(&pattern).bind(&pattern)
    .fetch_all(pool).await.map_err(|e| e.to_string())?;

    for (id, word, reading, meaning) in vocab_matches {
        results.push(SearchResult {
            id, content_type: "vocabulary".into(), title: word,
            subtitle: format!("{} — {}", reading, meaning), match_field: "word/reading/meaning".into(),
        });
    }

    let grammar_matches = sqlx::query_as::<_, (String, String, String)>(
        "SELECT id, pattern, meaning FROM grammar WHERE pattern LIKE ?1 OR meaning LIKE ?2 OR explanation LIKE ?3 LIMIT 10",
    )
    .bind(&pattern).bind(&pattern).bind(&pattern)
    .fetch_all(pool).await.map_err(|e| e.to_string())?;

    for (id, pattern, meaning) in grammar_matches {
        results.push(SearchResult {
            id, content_type: "grammar".into(), title: pattern,
            subtitle: meaning, match_field: "pattern/meaning".into(),
        });
    }

    let kanji_matches = sqlx::query_as::<_, (String, String, String)>(
        "SELECT id, character, meaning FROM kanji WHERE character LIKE ?1 OR meaning LIKE ?2 OR onyomi LIKE ?3 OR kunyomi LIKE ?4 LIMIT 10",
    )
    .bind(&pattern).bind(&pattern).bind(&pattern).bind(&pattern)
    .fetch_all(pool).await.map_err(|e| e.to_string())?;

    for (id, character, meaning) in kanji_matches {
        results.push(SearchResult {
            id, content_type: "kanji".into(), title: character,
            subtitle: meaning, match_field: "character/meaning".into(),
        });
    }

    Ok(results)
}
