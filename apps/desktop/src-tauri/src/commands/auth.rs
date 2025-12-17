
use tauri::{command, State, AppHandle};
use serde::{Serialize, Deserialize};
use machine_uid::get as get_machine_uid;
use keyring::Entry;
use reqwest::Client;
use crate::core::auth::pkce::PkceState;

#[cfg(debug_assertions)]
const API_BASE: &str = "http://localhost:3000";

#[cfg(not(debug_assertions))]
const API_BASE: &str = "https://YOUR_PRODUCTION_DOMAIN.com";

// ============================================================================
// PKCE Authentication Commands
// ============================================================================

#[derive(Serialize)]
pub struct InitiateLoginResponse {
    success: bool,
    message: String,
}

/// Initiate PKCE login flow
/// Generates proof, stores verifier in memory, opens browser with challenge
#[command]
pub async fn initiate_login(
    pkce_state: State<'_, PkceState>,
    app: AppHandle,
) -> Result<InitiateLoginResponse, String> {
    println!("[PKCE] Initiating login flow...");
    
    // Generate PKCE proof (verifier + challenge)
    let (_verifier, challenge) = pkce_state.generate_proof()
        .map_err(|e| format!("Failed to generate PKCE proof: {}", e))?;
    
    println!("[PKCE] Generated challenge (verifier stored in memory)");
    
    // Construct auth URL with challenge
    let auth_url = format!("{}/auth/device?challenge={}", API_BASE, challenge);
    println!("[PKCE] Opening browser to: {}", auth_url);
    
    // Open system browser
    if let Err(e) = opener::open(&auth_url) {
        return Err(format!("Failed to open browser: {}", e));
    }
    
    Ok(InitiateLoginResponse {
        success: true,
        message: "Browser opened. Please complete authentication.".to_string(),
    })
}

#[derive(Serialize, Deserialize)]
struct ExchangeRequest {
    code: String,
    verifier: String,
}

#[derive(Serialize, Deserialize)]
struct ExchangeResponse {
    token: String,
}

#[derive(Serialize)]
pub struct ExchangeTokenResponse {
    success: bool,
    token: Option<String>,
    message: String,
}

/// Exchange auth code for session token using PKCE verifier
#[command]
pub async fn exchange_token(
    auth_code: String,
    pkce_state: State<'_, PkceState>,
) -> Result<ExchangeTokenResponse, String> {
    println!("[PKCE] Exchanging auth code for token...");
    
    // Retrieve and consume the verifier (one-time use)
    let verifier = pkce_state.get_and_clear_verifier()
        .map_err(|e| format!("Failed to retrieve verifier: {}", e))?;
    
    println!("[PKCE] Retrieved verifier from memory");
    
    // Prepare exchange request
    let client = Client::new();
    let url = format!("{}/api/auth/exchange", API_BASE);
    
    let request_body = ExchangeRequest {
        code: auth_code.clone(),
        verifier,
    };
    
    println!("[PKCE] Sending exchange request to: {}", url);
    
    // Send exchange request
    let res = client.post(&url)
        .json(&request_body)
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;
    
    let status = res.status();
    println!("[PKCE] Exchange response status: {}", status);
    
    if !status.is_success() {
        let error_text = res.text().await.unwrap_or_else(|_| "Unknown error".to_string());
        return Err(format!("Exchange failed ({}): {}", status, error_text));
    }
    
    // Parse response
    let exchange_response: ExchangeResponse = res.json().await
        .map_err(|e| format!("Failed to parse response: {}", e))?;
    
    let token = exchange_response.token;
    println!("[PKCE] Received session token");
    
    // Save token to keyring
    let entry = Entry::new("prodwidgets", "current_user")
        .map_err(|e| format!("Failed to create keyring entry: {}", e))?;
    
    entry.set_password(&token)
        .map_err(|e| format!("Failed to save token to keyring: {}", e))?;
    
    println!("[PKCE] Token saved to keyring");
    
    Ok(ExchangeTokenResponse {
        success: true,
        token: Some(token),
        message: "Authentication successful".to_string(),
    })
}

// ============================================================================
// Legacy Handshake (Deprecated - kept for backward compatibility)
// ============================================================================

#[derive(Serialize)]
pub struct HandshakeResponse {
    success: bool,
    token: Option<String>,
}

#[deprecated(note = "Use PKCE flow (initiate_login + exchange_token) instead")]
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
