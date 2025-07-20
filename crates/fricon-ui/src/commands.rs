use super::AppState;

use tauri::{AppHandle, State, ipc::Invoke};
use tauri_plugin_dialog::DialogExt;

use fricon::{client::Client, paths::WorkDirectory};

#[tauri::command]
async fn select_workspace(app: AppHandle, state: State<'_, AppState>) -> Result<(), String> {
    let mut client_state = state.client.lock().await;
    if client_state.is_some() {
        return Err("Already connected".to_string());
    }
    let path = app
        .dialog()
        .file()
        .blocking_pick_folder()
        .ok_or("No folder selected")?
        .into_path()
        .unwrap();
    let work_directory = WorkDirectory::new(&path).unwrap();
    let client = Client::connect(work_directory.ipc_file())
        .await
        .map_err(|e| e.to_string())?;
    client_state.replace(client);
    Ok(())
}

#[tauri::command]
async fn get_connection_status(state: State<'_, AppState>) -> Result<bool, ()> {
    Ok(state.client.lock().await.as_ref().is_some())
}

pub fn invoke_handler() -> impl Fn(Invoke) -> bool {
    tauri::generate_handler![select_workspace, get_connection_status]
}
