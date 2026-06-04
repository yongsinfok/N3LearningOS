use crate::models::{ImportFileResult, ImportMode};

#[tauri::command]
pub async fn import_vocabulary_from_file(
    _file_path: String,
    _mode: ImportMode,
) -> Result<ImportFileResult, String> {
    Err("Not yet implemented".to_string())
}

#[tauri::command]
pub async fn import_grammar_from_file(
    _file_path: String,
    _mode: ImportMode,
) -> Result<ImportFileResult, String> {
    Err("Not yet implemented".to_string())
}

#[tauri::command]
pub async fn import_kanji_from_file(
    _file_path: String,
    _mode: ImportMode,
) -> Result<ImportFileResult, String> {
    Err("Not yet implemented".to_string())
}

#[tauri::command]
pub async fn import_reading_from_file(
    _file_path: String,
    _mode: ImportMode,
) -> Result<ImportFileResult, String> {
    Err("Not yet implemented".to_string())
}

#[tauri::command]
pub async fn import_listening_from_file(
    _file_path: String,
    _mode: ImportMode,
) -> Result<ImportFileResult, String> {
    Err("Not yet implemented".to_string())
}
