use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaptureResult {
    pub text: String,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslateRequest {
    pub text: String,
    pub source_lang: Option<String>,
    pub target_lang: String,
    pub provider: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslateResponse {
    pub translated: String,
    pub detected_source_lang: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConvertCurrencyRequest {
    pub amount: f64,
    pub from: String,
    pub to: String,
    pub date: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConvertCurrencyResponse {
    pub result: f64,
    pub rate: f64,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenWidgetRequest {
    pub widget: String,
    pub payload: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogRequest {
    pub level: String,
    pub message: String,
}


// New types for unified command palette
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActionType {
    // Translation actions - 26 languages
    TranslateEn,
    TranslateZh,
    TranslateEs,
    TranslateFr,
    TranslateDe,
    TranslateAr,
    TranslatePt,
    TranslateRu,
    TranslateJa,
    TranslateHi,
    TranslateIt,
    TranslateNl,
    TranslatePl,
    TranslateTr,
    TranslateHy,
    TranslateFa,
    TranslateVi,
    TranslateId,
    TranslateKo,
    TranslateBn,
    TranslateUr,
    TranslateTh,
    TranslateSv,
    TranslateDa,
    TranslateFi,
    TranslateHu,
    
    // Currency conversion actions - 10 currencies
    ConvertUsd,
    ConvertEur,
    ConvertGbp,
    ConvertJpy,
    ConvertAud,
    ConvertCad,
    ConvertChf,
    ConvertCny,
    ConvertInr,
    ConvertMxn,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandItem {
    pub id: String,
    pub label: String,
    pub description: Option<String>,
    pub action_type: Option<ActionType>,
    pub widget_type: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ExecuteActionRequest {
    pub action_type: ActionType,
    pub params: serde_json::Value,
}

#[derive(Debug, Clone, Serialize)]
pub struct ExecuteActionResponse {
    pub result: String,
    pub metadata: Option<serde_json::Value>,
}

// Cursor positioning types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CursorPosition {
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScreenBounds {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowPosition {
    pub x: f64,
    pub y: f64,
}
