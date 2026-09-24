use reqwest::Client;
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
struct AuthResponse {
    access_token: String,
    expires_in: u64,
    refresh_expires_in: u64,
    refresh_token: String,
    token_type: String,
    #[serde(rename = "not-before-policy")]
    not_before_policy: i64,
    session_state: String,
    scope: String,
}

#[tauri::command]
pub async fn get_copernicus_token(username: &str, password: &str) -> Result<String, String> {
    let url =
        "https://identity.dataspace.copernicus.eu/auth/realms/CDSE/protocol/openid-connect/token";

    let client = Client::new();

    let mut params = HashMap::new();
    params.insert("client_id", "cdse-public");
    params.insert("username", username);
    params.insert("password", password);
    params.insert("grant_type", "password");

    println!("Fetching token from CDSE Keycloak");
    let response = client
        .post(url)
        .form(&params)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if response.status().is_success() {
        let auth_data: AuthResponse = response.json().await.map_err(|e| e.to_string())?;
        println!(
            "Successfully retrieved token! (Expires in {} seconds)",
            auth_data.expires_in
        );
        println!("Access Token: {}", &auth_data.access_token);
        Ok(auth_data.access_token)
    } else {
        let error_text = response.text().await.map_err(|e| e.to_string())?;
        Err(format!("Authentication failed: {}", error_text))
    }
}
