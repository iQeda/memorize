mod commands;
mod error;
mod progress;
mod state;

use state::AppState;
use std::sync::atomic::Ordering;
use tauri::menu::{
    AboutMetadataBuilder, MenuBuilder, MenuItemBuilder, SubmenuBuilder,
};
use tauri::{Emitter, Manager};

const QUIT_MENU_ID: &str = "memorize_quit";

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "memorize=debug,anki=debug,warn".into()),
        )
        .init();

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None,
        ))
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .manage(AppState::default())
        .setup(|app| {
            // Tauri 2 on macOS routes the default menu's Quit item directly
            // to NSApplicationTerminate, which bypasses both
            // RunEvent::ExitRequested and WindowEvent::CloseRequested. To
            // keep ⌘Q preventable, rebuild the macOS menu with our own Quit
            // item that just emits an event the frontend listens for.
            let app_name = app.package_info().name.clone();
            let quit_label = format!("Quit {}", app_name);
            let quit_item = MenuItemBuilder::new(&quit_label)
                .id(QUIT_MENU_ID)
                .accelerator("CmdOrCtrl+Q")
                .build(app)?;
            let app_submenu = SubmenuBuilder::new(app, &app_name)
                .about(Some(AboutMetadataBuilder::new().build()))
                .separator()
                .services()
                .separator()
                .hide()
                .hide_others()
                .show_all()
                .separator()
                .item(&quit_item)
                .build()?;
            let edit_submenu = SubmenuBuilder::new(app, "Edit")
                .undo()
                .redo()
                .separator()
                .cut()
                .copy()
                .paste()
                .select_all()
                .build()?;
            let window_submenu = SubmenuBuilder::new(app, "Window")
                .minimize()
                .maximize()
                .separator()
                .close_window()
                .build()?;
            let menu = MenuBuilder::new(app)
                .items(&[&app_submenu, &edit_submenu, &window_submenu])
                .build()?;
            app.set_menu(menu)?;
            Ok(())
        })
        .on_menu_event(|app, event| {
            if event.id().as_ref() == QUIT_MENU_ID {
                let state = app.state::<AppState>();
                if state.allow_exit.load(Ordering::SeqCst) {
                    return;
                }
                let _ = app.emit("memorize://exit-requested", ());
            }
        })
        .on_window_event(|window, event| {
            // Window X button. On macOS this is normally just "hide window"
            // but for our single-window app we treat it like a quit so the
            // shutdown sync runs.
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                let app = window.app_handle();
                let state = app.state::<AppState>();
                if !state.allow_exit.load(Ordering::SeqCst) {
                    api.prevent_close();
                    let _ = app.emit("memorize://exit-requested", ());
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::app::confirm_exit,
            commands::collection::open_collection,
            commands::collection::close_collection,
            commands::collection::is_open,
            commands::collection::collection_info,
            commands::decks::list_decks,
            commands::decks::create_deck,
            commands::decks::rename_deck,
            commands::decks::delete_deck,
            commands::decks::deck_stats,
            commands::decks::deck_graph_stats,
            commands::cards::list_cards,
            commands::cards::list_due_cards,
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
            commands::notes::list_notetypes,
            commands::notes::get_note,
            commands::notes::add_note,
            commands::notes::update_note,
            commands::notes::delete_notes,
        ])
        .build(tauri::generate_context!())
        .expect("error while running memorize")
        .run(|app_handle, event| {
            // Backstop for any other "user wants to quit" path (e.g. a
            // platform-specific exit signal). The custom Quit menu and the
            // window close button already route through their own
            // handlers and `confirm_exit`.
            if let tauri::RunEvent::ExitRequested { api, .. } = event {
                let state = app_handle.state::<AppState>();
                if !state.allow_exit.load(Ordering::SeqCst) {
                    api.prevent_exit();
                    let _ = app_handle.emit("memorize://exit-requested", ());
                }
            }
        });
}
