pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![get_random_array, greet])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[tauri::command]
fn get_random_array() -> Vec<i32> {
    (0..10).collect()
}

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {name}!")
}
