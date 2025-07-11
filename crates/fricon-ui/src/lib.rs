#![allow(clippy::needless_pass_by_value)]
use std::sync::Mutex;

use tauri::State;

#[derive(Default)]
struct AppState {
    client: Option<fricon::client::Client>,
    workspace_path: Mutex<Option<String>>,
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
        .invoke_handler(tauri::generate_handler![
            set_workspace_path,
            get_connection_status,
            greet
        ])
        .manage(AppState::new())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[tauri::command]
fn set_workspace_path(path: &str, state: State<'_, AppState>) -> Result<(), &'static str> {
    let mut workspace_path = state.workspace_path.lock().unwrap();
    if workspace_path.is_some() {
        Err("Workspace path already set")
    } else {
        *workspace_path = Some(path.to_string());
        Ok(())
    }
}

#[tauri::command]
fn get_connection_status(state: State<'_, AppState>) -> &'static str {
    match state.client {
        Some(_) => "Ok",
        None => "Disconnected",
    }
}

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {name}!")
}
