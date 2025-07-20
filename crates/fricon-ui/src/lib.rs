#![allow(clippy::needless_pass_by_value, clippy::used_underscore_binding)]

use tauri::async_runtime::Mutex;

mod commands;

#[derive(Default)]
struct AppState {
    client: Mutex<Option<fricon::client::Client>>,
}

impl AppState {
    fn new() -> Self {
        Self::default()
    }
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(commands::invoke_handler())
        .manage(AppState::new())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
