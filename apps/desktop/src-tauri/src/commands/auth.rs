
use tauri::command;
use serde::Serialize;
use machine_uid::get as get_machine_uid;
use keyring::Entry;
use reqwest::Client;

#[cfg(debug_assertions)]
const API_BASE: &str = "http://localhost:3000";

#[cfg(not(debug_assertions))]
const API_BASE: &str = "https://YOUR_PRODUCTION_DOMAIN.com";

#[derive(Serialize)]
pub struct HandshakeResponse {
    success: bool,
    token: Option<String>,
}

#[command]
pub async fn perform_handshake(otp: String) -> Result<HandshakeResponse, String> {
    let hwid = get_machine_uid().map_err(|e| e.to_string())?;
    
    let client = Client::new();
    let url = format!("{}/api/auth/handshake", API_BASE);
    
    let res = client.post(&url)
        .json(&serde_json::json!({
            "otp": otp,
            "hardware_id": hwid
        }))
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if res.status().is_success() {
        let body: serde_json::Value = res.json().await.map_err(|e| e.to_string())?;
        if let Some(token) = body.get("token").and_then(|t| t.as_str()) {
            // Save to keyring
            let entry = Entry::new("prodwidgets", "current_user").map_err(|e| e.to_string())?;
            entry.set_password(token).map_err(|e| e.to_string())?;
            
            return Ok(HandshakeResponse { success: true, token: Some(token.to_string()) });
        }
    }
    
    Err("Handshake failed".to_string())
}
