pub mod bearer_token_ops;
pub mod fetch_image;
pub mod get_height_map;
pub mod execute_blender_shit;
pub mod heatmap_ops;
mod generate_auth_token;
use bearer_token_ops::{get_bearer_token, new_bearer_token};
use fetch_image::fetch_sentinel_patch;
use generate_auth_token::get_copernicus_token;
use get_height_map::get_height_map;
use execute_blender_shit::execute_blender_shit;
use heatmap_ops::{compute_hallucination_analysis, generate_noise_variants, run_super_res_single};

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
            fetch_sentinel_patch,
            get_height_map,
            execute_blender_shit,
            generate_noise_variants,
            run_super_res_single,
            compute_hallucination_analysis
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
