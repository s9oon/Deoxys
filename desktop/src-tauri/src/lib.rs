// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/

use shared::Hello;

const BACKEND_URL: &str = match option_env!("BACKEND_URL") {
    Some(url) => url,
    None => "http://localhost:3000",
};

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
async fn get_hello() -> Result<Hello, String> {
    reqwest::get(format!("{BACKEND_URL}/"))
        .await
        .map_err(|e| e.to_string())?
        .json::<Hello>()          // JSON → shared::Hello; fails if the shape doesn't match
        .await
        .map_err(|e| e.to_string())
}

// /health has no body, only a status code, so there's no shared type for it.
#[tauri::command]
async fn check_health() -> Result<(), String> {
    reqwest::get(format!("{BACKEND_URL}/health"))
        .await
        .map_err(|e| e.to_string())?
        .error_for_status()       // 503 etc. → Err; 200 → Ok
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet, get_hello, check_health])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
