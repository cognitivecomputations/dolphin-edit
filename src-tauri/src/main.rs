// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod indexing_service;
mod cache_manager;
mod state;
mod commands;

fn main() {
    let app_state = state::AppState::new();

    tauri::Builder::default()
        .manage(app_state) // Add AppState to Tauri's managed state
        .invoke_handler(tauri::generate_handler![
            commands::open_file,
            commands::get_total_lines,
            commands::get_lines,
            commands::get_line_content,
            commands::get_indexing_status
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
