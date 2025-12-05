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
pub struct ConvertUnitsRequest {
    pub amount: f64,
    pub from_unit: String,
    pub to_unit: String,
    pub material: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConvertUnitsResponse {
    pub result: f64,
    pub from_unit: String,
    pub to_unit: String,
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
    
    // Unit conversion actions - Length (8)
    ConvertToMM,
    ConvertToCM,
    ConvertToM,
    ConvertToKM,
    ConvertToIN,
    ConvertToFT,
    ConvertToYD,
    ConvertToMI,
    
    // Unit conversion actions - Mass (5)
    ConvertToMG,
    ConvertToG,
    ConvertToKG,
    ConvertToOZ,
    ConvertToLB,
    
    // Unit conversion actions - Volume (7)
    ConvertToML,
    ConvertToL,
    ConvertToFlOz,
    ConvertToCup,
    ConvertToPint,
    ConvertToQuart,
    ConvertToGal,
    
    // Unit conversion actions - Temperature (3)
    ConvertToC,
    ConvertToF,
    ConvertToK,
    
    // Unit conversion actions - Speed (4)
    ConvertToMS,
    ConvertToKMH,
    ConvertToMPH,
    ConvertToKnot,
    
    // Cross-category conversions - Volume to Mass (4)
    ConvertVolToG,
    ConvertVolToKG,
    ConvertVolToOZ,
    ConvertVolToLB,
    
    // Cross-category conversions - Mass to Volume (7)
    ConvertMassToML,
    ConvertMassToL,
    ConvertMassToFlOz,
    ConvertMassToCup,
    ConvertMassToPint,
    ConvertMassToQuart,
    ConvertMassToGal,
    
    // Time zone conversion - polymorphic variant carrying timezone ID
    ConvertTime(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConvertTimeRequest {
    pub time_input: String,
    pub target_timezone: String,
    pub source_timezone: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub matched_keyword: Option<String>,  // NEW: Which keyword triggered timezone detection
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConvertTimeResponse {
    pub source_time: String,
    pub target_time: String,
    pub offset_description: String,
    pub source_timezone: String,
    pub target_timezone: String,
    
    // Enhanced fields
    pub target_utc_offset: String,        // e.g., "UTC-04:00"
    pub target_zone_abbr: String,          // e.g., "EDT" (DST-aware)
    pub relative_offset: String,           // e.g., "+3h 0m"
    pub date_change_indicator: Option<String>, // "Next day" / "Previous day"
    pub source_zone_abbr: String,          // e.g., "KST"
    pub source_utc_offset: String,         // e.g., "UTC+09:00"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimezoneInfo {
    pub label: String,
    pub iana_id: String,
    pub keywords: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedTimeInput {
    pub time_input: String,
    pub source_timezone: Option<String>,  // IANA ID or None for Local
    #[serde(skip_serializing_if = "Option::is_none")]
    pub matched_keyword: Option<String>,  // NEW: Which keyword triggered timezone detection
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandItem {
    pub id: String,
    pub label: String,
    pub description: Option<String>,
    pub action_type: Option<ActionType>,
    pub widget_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<crate::core::context::category::ContextCategory>,
}

// Forward declaration to avoid circular dependency
// The category module will be defined in context/category.rs

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
