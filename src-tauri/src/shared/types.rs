use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/types/bindings.ts")]
pub struct CaptureResult {
    pub text: String,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/types/bindings.ts")]
pub struct TranslateRequest {
    pub text: String,
    pub source_lang: Option<String>,
    pub target_lang: String,
    pub provider: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/types/bindings.ts")]
pub struct TranslateResponse {
    pub translated: String,
    pub detected_source_lang: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/types/bindings.ts")]
pub struct ConvertCurrencyRequest {
    #[ts(type = "string")]
    pub amount: String,
    pub from: String,
    pub to: String,
    pub date: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/types/bindings.ts")]
pub struct ConvertCurrencyResponse {
    pub result: String,
    pub rate: String,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/types/bindings.ts")]
pub struct ConvertUnitsRequest {
    pub amount: f64,
    pub from_unit: String,
    pub to_unit: String,
    pub material: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/types/bindings.ts")]
pub struct ConvertUnitsResponse {
    pub result: f64,
    pub from_unit: String,
    pub to_unit: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/types/bindings.ts")]
pub struct OpenWidgetRequest {
    pub widget: String,
    #[ts(type = "any")]
    pub payload: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/types/bindings.ts")]
pub struct LogRequest {
    pub level: String,
    pub message: String,
}


// New types for unified command palette
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "../../src/types/bindings.ts")]
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
    
    // Definition lookup actions
    FindSynonyms,
    FindAntonyms,
    BriefDefinition,
    
    // Clipboard actions
    ClearClipboardHistory,
    PauseClipboard,
    ResumeClipboard,

    // Text analysis actions
    CountWords,
    CountChars,
    ReadingTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/types/bindings.ts")]
pub struct ConvertTimeRequest {
    pub time_input: String,
    pub target_timezone: String,
    pub source_timezone: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub matched_keyword: Option<String>,  // NEW: Which keyword triggered timezone detection
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/types/bindings.ts")]
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

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/types/bindings.ts")]
pub struct TimezoneInfo {
    pub label: String,
    pub iana_id: String,
    pub keywords: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/types/bindings.ts")]
pub struct ParsedTimeInput {
    pub time_input: String,
    pub source_timezone: Option<String>,  // IANA ID or None for Local
    #[serde(skip_serializing_if = "Option::is_none")]
    pub matched_keyword: Option<String>,  // NEW: Which keyword triggered timezone detection
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/types/bindings.ts")]
pub struct LookupDefinitionRequest {
    pub word: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/types/bindings.ts")]
pub struct LookupDefinitionResponse {
    pub word: String,
    pub phonetic: Option<String>,
    pub definitions: Vec<DefinitionEntry>,
    pub synonyms: Vec<String>,
    pub antonyms: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/types/bindings.ts")]
pub struct DefinitionEntry {
    pub part_of_speech: String,
    pub definition: String,
    pub example: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/types/bindings.ts")]
pub struct TextAnalysisRequest {
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/types/bindings.ts")]
pub struct TextAnalysisResponse {
    pub word_count: usize,
    pub char_count: usize,
    pub char_count_no_spaces: usize,
    pub grapheme_count: usize,
    pub line_count: usize,
    pub reading_time_sec: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/types/bindings.ts")]
pub struct CommandItem {
    pub id: String,
    pub label: String,
    pub description: Option<String>,
    pub action_type: Option<ActionType>,
    pub widget_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<crate::core::context::category::ContextCategory>,
}

// OpenWidgetRequest removed (duplicate)

// Forward declaration to avoid circular dependency
// The category module will be defined in context/category.rs

use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/types/bindings.ts")]
pub struct ExecuteActionRequest {
    pub action_type: ActionType,
    #[ts(type = "any")]
    pub params: serde_json::Value,
}

/// Type of clipboard content
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, TS)]
#[serde(rename_all = "lowercase")]
#[ts(export, export_to = "../../src/types/bindings.ts")]
pub enum ClipboardItemType {
    Text,
    Html,
    Rtf,
    Image,
}

/// A single clipboard history item
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/types/bindings.ts")]
pub struct ClipboardHistoryItem {
    pub id: String,
    pub item_type: ClipboardItemType,
    pub content: String, // For images, this would be base64 or path
    pub preview: String, // Truncated preview for display
    #[ts(type = "string")]
    pub timestamp: DateTime<Utc>,
    pub source_app: Option<String>,
}

impl ClipboardHistoryItem {
    /// Create a new text clipboard item
    pub fn new_text(content: String, source_app: Option<String>) -> Self {
        let preview = if content.len() > 100 {
            format!("{}...", &content[..100])
        } else {
            content.clone()
        };

        Self {
            id: uuid::Uuid::new_v4().to_string(),
            item_type: ClipboardItemType::Text,
            content,
            preview,
            timestamp: Utc::now(),
            source_app,
        }
    }

    /// Create a new HTML clipboard item
    pub fn new_html(content: String, source_app: Option<String>) -> Self {
        // Strip HTML tags for preview (helper implementation needed or moved here? 
        // Better to implement stripping logic in core, but preview is part of data.
        // For shared types, we usually avoid complex logic. 
        // I will keep the method signature but maybe move `strip_html_tags` too or duplicate it? 
        // `strip_html_tags` is private in history.rs. I'll make it public there or move it to utils.
        // For now, I'll rely on a helper function or assume caller handles it? 
        // Actually, the `new_html` method in `history.rs` was a convenience constructor.
        // It's better to keep constructors in `history.rs` extension traits or similar, BUT
        // `ClipboardItem` is a data struct.
        // I will copy `strip_html_tags` logic here or move it to `text_utils` later. 
        // To be safe and quick, I'll implement a simple stripper here or remove constructors from shared type?
        // Structs in `shared/types.rs` are mostly DTOs. 
        // However, `monitor.rs` uses `new_text`, `new_html`.
        // I should keep constructors if possible or move them to `impl ClipboardHistoryItem` in `history.rs` via trait?
        // No, `impl` blocks can be anywhere. Using it here is fine.
        
        let preview = strip_html_tags(&content);
        let preview = if preview.len() > 100 {
            format!("{}...", &preview[..100])
        } else {
            preview
        };

        Self {
            id: uuid::Uuid::new_v4().to_string(),
            item_type: ClipboardItemType::Html,
            content,
            preview,
            timestamp: Utc::now(),
            source_app,
        }
    }

    /// Create a new image clipboard item
    pub fn new_image(content: String, source_app: Option<String>) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            item_type: ClipboardItemType::Image,
            content,
            preview: "[Image]".to_string(),
            timestamp: Utc::now(),
            source_app,
        }
    }
}

/// Simple HTML tag stripper for preview generation (Helper)
fn strip_html_tags(html: &str) -> String {
    let mut result = String::new();
    let mut in_tag = false;

    for c in html.chars() {
        match c {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => result.push(c),
            _ => {}
        }
    }

    result.trim().to_string()
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "../../src/types/bindings.ts")]
pub struct ExecuteActionResponse {
    pub result: String,
    #[ts(type = "any")]
    pub metadata: Option<serde_json::Value>,
}

// Cursor positioning types
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/types/bindings.ts")]
pub struct CursorPosition {
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/types/bindings.ts")]
pub struct ScreenBounds {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/types/bindings.ts")]
pub struct WindowPosition {
    pub x: f64,
    pub y: f64,
}
