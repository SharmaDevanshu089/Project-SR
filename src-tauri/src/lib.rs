pub mod bearer_token_ops;
pub mod fetch_image;
mod generate_auth_token;
use bearer_token_ops::{get_bearer_token, new_bearer_token};
use fetch_image::fetch_sentinel_patch;
use generate_auth_token::get_copernicus_token;
// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            get_copernicus_token,
            new_bearer_token,
            get_bearer_token,
            fetch_sentinel_patch
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
