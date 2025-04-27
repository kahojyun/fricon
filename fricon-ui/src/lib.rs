pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_log::Builder::new().build())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![get_random_array])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[tauri::command]
fn get_random_array() -> Vec<i32> {
    use rand::Rng;
    let mut rng = rand::rng();
    let mut array = Vec::new();

    for _ in 0..10 {
        array.push(rng.random_range(0..100));
    }

    array
}
