pub mod commands;
pub mod database;
pub mod fsrs;
pub mod importers;
pub mod models;
pub mod services;

use std::path::PathBuf;

fn get_app_dir() -> PathBuf {
    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .unwrap_or_else(|_| ".".to_string());
    let app_dir = PathBuf::from(home).join(".n3-learning");
    std::fs::create_dir_all(&app_dir).ok();
    app_dir
}

fn get_content_dir() -> PathBuf {
    let exe = std::env::current_exe().unwrap_or_default();
    let exe_dir = exe.parent().unwrap_or(&PathBuf::from(".")).to_path_buf();
    exe_dir.join("content")
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|_app| {
            let app_dir = get_app_dir();
            let content_dir = get_content_dir();

            tauri::async_runtime::block_on(async move {
                crate::database::init_database(app_dir.clone())
                    .await
                    .expect("Failed to initialize database");

                if content_dir.join("manifest.json").exists() {
                    match crate::importers::run_initial_import(&content_dir) {
                        Ok(result) => {
                            println!(
                                "Import: {} vocabulary, {} grammar, {} kanji, {} reading, {} listening, {} flashcards",
                                result.vocabulary_imported,
                                result.grammar_imported,
                                result.kanji_imported,
                                result.reading_imported,
                                result.listening_imported,
                                result.flashcards_created
                            );
                        }
                        Err(e) => {
                            eprintln!("Import error: {}", e);
                        }
                    }
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::vocabulary::list_vocabulary,
            commands::vocabulary::get_vocabulary_detail,
            commands::vocabulary::get_due_flashcards,
            commands::vocabulary::review_flashcard,
            commands::dashboard::get_dashboard,
            commands::search::global_search,
            commands::notes::list_notes,
            commands::notes::save_note,
            commands::notes::delete_note,
            commands::reading::list_reading,
            commands::reading::get_reading_detail,
            commands::reading::lookup_word,
            commands::listening::list_listening,
            commands::listening::get_listening_detail,
            commands::listening::get_audio_data,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
