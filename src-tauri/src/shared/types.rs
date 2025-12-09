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
    pub formatted_result: String,
    pub from_unit: String,
    pub to_unit: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/types/bindings.ts")]
pub struct ConvertUnitPayload {
    pub value: f64,
    pub from_unit: String,
    pub target_unit: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/types/bindings.ts")]
pub struct ParseUnitResponse {
    pub amount: f64,
    pub unit: String,
    pub category: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/types/bindings.ts")]
pub struct GetUnitsResponse {
    pub units: Vec<UnitDTO>,
}

// Rich Unit Data Transfer Object for frontend
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/types/bindings.ts")]
pub struct UnitDTO {
    pub id: String,       // Unit symbol (e.g., "m", "kg")
    pub label: String,    // Display name (e.g., "Meters", "Kilograms")
    pub category: String, // Category (e.g., "Length", "Mass")
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


// Action types for command palette and widget actions
// Using adjacently tagged serialization for frontend compatibility
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(tag = "type", content = "payload")]
#[ts(export, export_to = "../../src/types/bindings.ts")]
pub enum ActionType {
    // ===== NEW: Intent-based variants (Phase 1) =====
    /// Handles ALL translations with a single variant
    /// Replaces: TranslateEn, TranslateZh, TranslateEs, etc.
    Translate(TranslatePayload),
    
    // ===== OLD: Keep temporarily for backward compatibility =====
    // Translation actions - 26 languages (to be removed in Phase 4)
    #[deprecated(note = "Use Translate(TranslatePayload) instead")]
    TranslateEn,
    #[deprecated(note = "Use Translate(TranslatePayload) instead")]
    TranslateZh,
    #[deprecated(note = "Use Translate(TranslatePayload) instead")]
    TranslateEs,
    #[deprecated(note = "Use Translate(TranslatePayload) instead")]
    TranslateFr,
    #[deprecated(note = "Use Translate(TranslatePayload) instead")]
    TranslateDe,
    #[deprecated(note = "Use Translate(TranslatePayload) instead")]
    TranslateAr,
    #[deprecated(note = "Use Translate(TranslatePayload) instead")]
    TranslatePt,
    #[deprecated(note = "Use Translate(TranslatePayload) instead")]
    TranslateRu,
    #[deprecated(note = "Use Translate(TranslatePayload) instead")]
    TranslateJa,
    #[deprecated(note = "Use Translate(TranslatePayload) instead")]
    TranslateHi,
    #[deprecated(note = "Use Translate(TranslatePayload) instead")]
    TranslateIt,
    #[deprecated(note = "Use Translate(TranslatePayload) instead")]
    TranslateNl,
    #[deprecated(note = "Use Translate(TranslatePayload) instead")]
    TranslatePl,
    #[deprecated(note = "Use Translate(TranslatePayload) instead")]
    TranslateTr,
    #[deprecated(note = "Use Translate(TranslatePayload) instead")]
    TranslateHy,
    #[deprecated(note = "Use Translate(TranslatePayload) instead")]
    TranslateFa,
    #[deprecated(note = "Use Translate(TranslatePayload) instead")]
    TranslateVi,
    #[deprecated(note = "Use Translate(TranslatePayload) instead")]
    TranslateId,
    #[deprecated(note = "Use Translate(TranslatePayload) instead")]
    TranslateKo,
    #[deprecated(note = "Use Translate(TranslatePayload) instead")]
    TranslateBn,
    #[deprecated(note = "Use Translate(TranslatePayload) instead")]
    TranslateUr,
    #[deprecated(note = "Use Translate(TranslatePayload) instead")]
    TranslateTh,
    #[deprecated(note = "Use Translate(TranslatePayload) instead")]
    TranslateSv,
    #[deprecated(note = "Use Translate(TranslatePayload) instead")]
    TranslateDa,
    #[deprecated(note = "Use Translate(TranslatePayload) instead")]
    TranslateFi,
    #[deprecated(note = "Use Translate(TranslatePayload) instead")]
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
    
    // Generic unit conversion action with payload
    ConvertUnit { target: String },
    
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

// ===== NEW: Payload Structures (Phase 1) =====

/// Payload for translation actions
/// Carries target language code and optional source language
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/types/bindings.ts")]
pub struct TranslatePayload {
    /// Target language code (e.g., "en", "es", "zh")
    pub target_lang: String,
    /// Optional source language code (None = auto-detect)
    pub source_lang: Option<String>,
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
