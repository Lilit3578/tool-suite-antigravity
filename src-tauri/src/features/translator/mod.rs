pub mod service;
pub mod types;

use service::TranslatorService;
use types::{TranslationRequest, TranslationResponse, TranslatorError};

#[tauri::command]
pub async fn translate_text(request: TranslationRequest) -> Result<TranslationResponse, String> {
    let service = TranslatorService::new().map_err(|e| e.to_string())?;
    let translated = service.translate(request).await.map_err(|e| e.to_string())?;
    Ok(TranslationResponse {
        translated,
        detected: None,
        cached: false,
    })
}
