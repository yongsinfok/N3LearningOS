pub mod manifest;
pub mod vocabulary;

use crate::models::ImportResult;
use std::path::Path;

pub fn run_initial_import(content_dir: &Path) -> Result<ImportResult, String> {
    let manifest = manifest::Manifest::load(content_dir)?;

    // Check already imported
    let current_version = get_imported_version()
        .map_err(|e| format!("Failed to check version: {}", e))?;

    if current_version.as_deref() == Some(&manifest.version) {
        return Ok(ImportResult {
            vocabulary_imported: 0,
            grammar_imported: 0,
            kanji_imported: 0,
            flashcards_created: 0,
            success: true,
        });
    }

    let vocab = vocabulary::import_vocabulary(content_dir)?;
    let vocab_count = vocabulary::save_vocabulary_to_db(&vocab)
        .map_err(|e| format!("DB insert failed: {}", e))?;
    let flashcard_count = vocabulary::create_vocab_flashcards()
        .map_err(|e| format!("Flashcard creation failed: {}", e))?;

    set_imported_version(&manifest.version)
        .map_err(|e| format!("Failed to save version: {}", e))?;

    Ok(ImportResult {
        vocabulary_imported: vocab_count,
        grammar_imported: 0,
        kanji_imported: 0,
        flashcards_created: flashcard_count,
        success: true,
    })
}

fn get_imported_version() -> Result<Option<String>, sqlx::Error> {
    let pool = crate::database::get_pool().map_err(|e| sqlx::Error::Protocol(e.to_string()))?;
    futures::executor::block_on(async {
        sqlx::query_scalar::<_, String>(
            "SELECT value FROM metadata WHERE key = 'content_version'",
        )
        .fetch_optional(pool)
        .await
    })
}

fn set_imported_version(version: &str) -> Result<(), sqlx::Error> {
    let pool = crate::database::get_pool().map_err(|e| sqlx::Error::Protocol(e.to_string()))?;
    futures::executor::block_on(async {
        sqlx::query(
            "INSERT INTO metadata (key, value) VALUES ('content_version', ?1) \
             ON CONFLICT(key) DO UPDATE SET value = ?2",
        )
        .bind(version)
        .bind(version)
        .execute(pool)
        .await?;
        Ok(())
    })
}
