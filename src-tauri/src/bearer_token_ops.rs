use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Serialize, Deserialize)]
struct TokenData {
    #[serde(alias = "token")]
    bearer_token: String,
}

fn config_path() -> &'static str {
    if Path::new("project-sr,config").exists() {
        "project-sr,config"
    } else {
        "project-sr.config"
    }
}

#[tauri::command]
pub fn new_bearer_token(bearer_token: String) -> Result<(), String> {
    let path = config_path();
    let data = TokenData { bearer_token };
    let json_str = match serde_json::to_string_pretty(&data) {
        Ok(json) => json,
        Err(e) => {
            let err = format!("Failed to serialize token to JSON: {}", e);
            println!("{}", err);
            return Err(err);
        }
    };
    match fs::write(path, json_str) {
        Ok(_) => {
            println!("Successfully wrote bearer token to {}", path);
            Ok(())
        }
        Err(e) => {
            let err = format!("Failed to write to {}: {}", path, e);
            println!("{}", err);
            Err(err)
        }
    }
}

#[tauri::command]
pub fn get_bearer_token() -> Result<String, String> {
    let path = config_path();
    let contents = match fs::read_to_string(path) {
        Ok(data) => data,
        Err(e) => {
            let err = format!("Failed to read {}: {}", path, e);
            println!("{}", err);
            return Err(err);
        }
    };
    let data: TokenData = match serde_json::from_str(&contents) {
        Ok(parsed) => parsed,
        Err(e) => {
            let err = format!("Failed to parse JSON from {}: {}", path, e);
            println!("{}", err);
            return Err(err);
        }
    };
    println!("Successfully retrieved bearer token from {}", path);
    Ok(data.bearer_token)
}
