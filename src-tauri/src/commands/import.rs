use crate::models::{ImportFileResult, ImportMode};
use std::path::Path;

#[tauri::command]
pub async fn import_vocabulary_from_file(
    file_path: String,
    mode: ImportMode,
) -> Result<ImportFileResult, String> {
    let path = Path::new(&file_path);
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");

    if ext == "apkg" {
        let results = crate::importers::user_import::import_apkg(path, mode)?;
        let mut total = ImportFileResult { imported: 0, overwritten: 0, errors: vec![], total_in_file: 0 };
        for r in results {
            total.imported += r.imported;
            total.overwritten += r.overwritten;
            total.total_in_file += r.total_in_file;
        }
        return Ok(total);
    }

    crate::importers::user_import::import_vocabulary_from_file(path, mode)
}

#[tauri::command]
pub async fn import_grammar_from_file(
    file_path: String,
    mode: ImportMode,
) -> Result<ImportFileResult, String> {
    let path = Path::new(&file_path);
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");

    if ext == "apkg" {
        let results = crate::importers::user_import::import_apkg(path, mode)?;
        let mut total = ImportFileResult { imported: 0, overwritten: 0, errors: vec![], total_in_file: 0 };
        for r in results {
            total.imported += r.imported;
            total.overwritten += r.overwritten;
            total.total_in_file += r.total_in_file;
        }
        return Ok(total);
    }

    crate::importers::user_import::import_grammar_from_file(path, mode)
}

#[tauri::command]
pub async fn import_kanji_from_file(
    file_path: String,
    mode: ImportMode,
) -> Result<ImportFileResult, String> {
    let path = Path::new(&file_path);
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");

    if ext == "apkg" {
        let results = crate::importers::user_import::import_apkg(path, mode)?;
        let mut total = ImportFileResult { imported: 0, overwritten: 0, errors: vec![], total_in_file: 0 };
        for r in results {
            total.imported += r.imported;
            total.overwritten += r.overwritten;
            total.total_in_file += r.total_in_file;
        }
        return Ok(total);
    }

    crate::importers::user_import::import_kanji_from_file(path, mode)
}

#[tauri::command]
pub async fn import_reading_from_file(
    file_path: String,
    mode: ImportMode,
) -> Result<ImportFileResult, String> {
    crate::importers::user_import::import_reading_from_file(Path::new(&file_path), mode)
}

#[tauri::command]
pub async fn import_listening_from_file(
    file_path: String,
    mode: ImportMode,
) -> Result<ImportFileResult, String> {
    crate::importers::user_import::import_listening_from_file(Path::new(&file_path), mode)
}
