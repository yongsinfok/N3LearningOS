pub mod manifest;
pub mod vocabulary;
pub mod grammar;
pub mod kanji;
pub mod reading;
pub mod listening;

use crate::models::ImportResult;
use std::path::Path;

pub fn run_initial_import(content_dir: &Path) -> Result<ImportResult, String> {
    let manifest = manifest::Manifest::load(content_dir)?;

    let current_version = get_imported_version()
        .map_err(|e| format!("Failed to check version: {}", e))?;

    if current_version.as_deref() == Some(&manifest.version) {
        return Ok(ImportResult {
            vocabulary_imported: 0,
            grammar_imported: 0,
            kanji_imported: 0,
            reading_imported: 0,
            listening_imported: 0,
            flashcards_created: 0,
            success: true,
        });
    }

    let vocab = vocabulary::import_vocabulary(content_dir)?;
    let vocab_count = vocabulary::save_vocabulary_to_db(&vocab)
        .map_err(|e| format!("DB insert failed: {}", e))?;
    let vocab_fc = vocabulary::create_vocab_flashcards()
        .map_err(|e| format!("Flashcard failed: {}", e))?;

    let grammar = grammar::import_grammar(content_dir)?;
    let grammar_count = grammar::save_grammar_to_db(&grammar)
        .map_err(|e| format!("Grammar DB insert failed: {}", e))?;
    let grammar_fc = grammar::create_grammar_flashcards()
        .map_err(|e| format!("Grammar flashcard failed: {}", e))?;

    let kanji = kanji::import_kanji(content_dir)?;
    let kanji_count = kanji::save_kanji_to_db(&kanji)
        .map_err(|e| format!("Kanji DB insert failed: {}", e))?;
    let kanji_fc = kanji::create_kanji_flashcards()
        .map_err(|e| format!("Kanji flashcard failed: {}", e))?;

    let reading = reading::import_reading(content_dir)?;
    let reading_count = reading::save_reading_to_db(&reading)
        .map_err(|e| format!("Reading DB insert failed: {}", e))?;

    let listening = listening::import_listening(content_dir)?;
    let listening_count = listening::save_listening_to_db(&listening)
        .map_err(|e| format!("Listening DB insert failed: {}", e))?;

    set_imported_version(&manifest.version)
        .map_err(|e| format!("Failed to save version: {}", e))?;

    let total_fc = vocab_fc + grammar_fc + kanji_fc;

    Ok(ImportResult {
        vocabulary_imported: vocab_count,
        grammar_imported: grammar_count,
        kanji_imported: kanji_count,
        reading_imported: reading_count,
        listening_imported: listening_count,
        flashcards_created: total_fc,
        success: true,
    })
}

fn get_imported_version() -> Result<Option<String>, sqlx::Error> {
    let pool = crate::database::get_pool().map_err(|e| sqlx::Error::Protocol(e.to_string()))?;
    futures::executor::block_on(async {
        sqlx::query_scalar::<_, String>("SELECT value FROM metadata WHERE key = 'content_version'")
            .fetch_optional(pool).await
    })
}

fn set_imported_version(version: &str) -> Result<(), sqlx::Error> {
    let pool = crate::database::get_pool().map_err(|e| sqlx::Error::Protocol(e.to_string()))?;
    futures::executor::block_on(async {
        sqlx::query("INSERT INTO metadata (key, value) VALUES ('content_version', ?1) ON CONFLICT(key) DO UPDATE SET value = ?2")
            .bind(version).bind(version).execute(pool).await?;
        Ok(())
    })
}
