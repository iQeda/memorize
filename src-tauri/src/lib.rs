mod commands;
mod error;
mod progress;
mod state;

use state::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "memorize=debug,warn".into()),
        )
        .init();

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .manage(AppState::default())
        .invoke_handler(tauri::generate_handler![
            commands::collection::open_collection,
            commands::collection::close_collection,
            commands::collection::is_open,
            commands::decks::list_decks,
            commands::cards::list_cards,
            commands::cards::list_due_cards,
            commands::cards::get_study_queue,
            commands::reviewer::get_card_render,
            commands::sync::sync_status,
            commands::sync::sync_login_cmd,
            commands::sync::sync_logout,
            commands::sync::sync_now,
            commands::sync::sync_full_upload,
            commands::sync::sync_full_download,
            commands::backup::export_colpkg,
            commands::backup::auto_backup,
            commands::backup::import_colpkg,
            commands::package::import_apkg,
            commands::package::export_all_apkg,
            commands::study::start_study,
            commands::study::get_next_card,
            commands::study::answer_card_now,
        ])
        .run(tauri::generate_context!())
        .expect("error while running memorize");
}
