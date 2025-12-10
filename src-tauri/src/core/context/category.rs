//! Context category detection and mapping
//!
//! Provides scalable categorization system for content detection and action mapping.
//! This serves as the single source of truth for category assignments.

use crate::shared::types::ActionType;
use regex::Regex;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// Context categories for content detection and action filtering
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, TS)]
#[ts(export, export_to = "../../src/types/bindings.ts")]
pub enum ContextCategory {
    /// Length measurements (meters, feet, kilometers, etc.)
    Length,
    /// Mass/weight measurements (kilograms, pounds, grams, etc.)
    Mass,
    /// Volume measurements (liters, gallons, milliliters, etc.)
    Volume,
    /// Temperature measurements (Celsius, Fahrenheit, Kelvin)
    Temperature,
    /// Speed measurements (km/h, mph, m/s, etc.)
    Speed,
    /// Currency amounts (USD, EUR, GBP, etc.)
    Currency,
    /// Text content (for translation)
    Text,
    /// Time measurements (hours, minutes, seconds) - for future scalability
    Time,
    /// General/uncategorized content
    General,
}

/// Detect content category from text using regex patterns
/// 
/// This is the scalable detection point - adding new categories only requires
/// adding a regex pattern here, not touching the action mapping.
pub fn detect_content_category(text: &str) -> Option<ContextCategory> {
    // Truncate input to prevent DoS attacks from large text
    const MAX_INPUT_LENGTH: usize = 1000;
    let truncated = if text.len() > MAX_INPUT_LENGTH {
        &text[..MAX_INPUT_LENGTH]
    } else {
        text
    };
    let text_lower = truncated.to_lowercase();
    
    // Length patterns: numbers followed by length units
    let length_patterns = vec![
        r"\d+\.?\d*\s*(mm|cm|m|km|in|inch|inches|ft|foot|feet|yd|yard|yards|mi|mile|miles|millimeter|millimeters|centimeter|centimeters|meter|meters|kilometer|kilometers)",
        r"(mm|cm|m|km|in|ft|yd|mi)\s*\d+\.?\d*",
    ];
    for pattern in length_patterns {
        if let Ok(re) = Regex::new(pattern) {
            if re.is_match(&text_lower) {
                return Some(ContextCategory::Length);
            }
        }
    }
    
    // Mass patterns: numbers followed by mass units
    let mass_patterns = vec![
        r"\d+\.?\d*\s*(mg|g|kg|oz|ounce|ounces|lb|lbs|pound|pounds|milligram|milligrams|gram|grams|kilogram|kilograms)",
        r"(mg|g|kg|oz|lb)\s*\d+\.?\d*",
    ];
    for pattern in mass_patterns {
        if let Ok(re) = Regex::new(pattern) {
            if re.is_match(&text_lower) {
                return Some(ContextCategory::Mass);
            }
        }
    }
    
    // Volume patterns: numbers followed by volume units
    let volume_patterns = vec![
        r"\d+\.?\d*\s*(ml|l|liter|liters|litre|litres|fl-oz|floz|fluid\s*ounce|fluid\s*ounces|cup|cups|gal|gallon|gallons|milliliter|milliliters)",
        r"(ml|l|fl-oz|cup|gal)\s*\d+\.?\d*",
    ];
    for pattern in volume_patterns {
        if let Ok(re) = Regex::new(pattern) {
            if re.is_match(&text_lower) {
                return Some(ContextCategory::Volume);
            }
        }
    }
    
    // Temperature patterns: numbers followed by temperature units
    let temp_patterns = vec![
        r"\d+\.?\d*\s*(c|celsius|f|fahrenheit|k|kelvin|°c|°f|°C|°F)",
        r"(c|f|k|celsius|fahrenheit|kelvin)\s*\d+\.?\d*",
    ];
    for pattern in temp_patterns {
        if let Ok(re) = Regex::new(pattern) {
            if re.is_match(&text_lower) {
                return Some(ContextCategory::Temperature);
            }
        }
    }
    
    // Speed patterns: numbers followed by speed units
    let speed_patterns = vec![
        r"\d+\.?\d*\s*(km/h|kmh|kph|mph|m/h|kilometers?\s*per\s*hour|miles?\s*per\s*hour)",
        r"(km/h|kmh|mph)\s*\d+\.?\d*",
    ];
    for pattern in speed_patterns {
        if let Ok(re) = Regex::new(pattern) {
            if re.is_match(&text_lower) {
                return Some(ContextCategory::Speed);
            }
        }
    }
    
    // Currency patterns: currency symbols or codes with numbers
    let currency_patterns = vec![
        r"[$€£¥]\s*\d+\.?\d*",
        r"\d+\.?\d*\s*(usd|eur|gbp|jpy|aud|cad|chf|cny|inr|mxn|dollar|dollars|euro|euros|pound|pounds|yen)",
        r"(usd|eur|gbp|jpy|aud|cad|chf|cny|inr|mxn)\s*\d+\.?\d*",
    ];
    for pattern in currency_patterns {
        if let Ok(re) = Regex::new(pattern) {
            if re.is_match(&text_lower) {
                return Some(ContextCategory::Currency);
            }
        }
    }
    
    // Time patterns: numbers followed by time units (for future scalability)
    let time_patterns = vec![
        r"\d+\.?\d*\s*(h|hr|hrs|hour|hours|m|min|mins|minute|minutes|s|sec|secs|second|seconds|ms|millisecond|milliseconds)",
        r"(h|hr|min|sec|ms)\s*\d+\.?\d*",
    ];
    for pattern in time_patterns {
        if let Ok(re) = Regex::new(pattern) {
            if re.is_match(&text_lower) {
                return Some(ContextCategory::Time);
            }
        }
    }
    
    // Text detection: if it contains words (not just numbers/symbols)
    // This is a fallback for translation actions
    if text.chars().any(|c| c.is_alphabetic()) && text.len() > 2 {
        return Some(ContextCategory::Text);
    }
    
    None
}

/// Map ActionType to ContextCategory
/// 
/// This is the SCALABLE mapping point - adding new actions only requires
/// adding a match arm here, not touching detection or frontend logic.
pub fn get_action_category(action: &ActionType) -> Option<ContextCategory> {
    match action {
        // Translation - NEW structured variant only
        ActionType::Translate(_) => Some(ContextCategory::Text),
        
        // Text analysis and definition - NEW structured variants only
        ActionType::AnalyzeText(_)
        | ActionType::DefinitionAction(_) => Some(ContextCategory::Text),
        
        // Currency - NEW structured variant only
        ActionType::ConvertCurrency(_) => Some(ContextCategory::Currency),
        
        // Generic unit conversion (already structured)
        ActionType::ConvertUnit { target } => get_unit_category(target),
        
        // Time conversion - NEW structured variant only
        ActionType::ConvertTimeAction(_) => Some(ContextCategory::Time),
        
        // Clipboard - NEW structured variant only
        ActionType::ClipboardAction(_) => Some(ContextCategory::General),
    }
}

/// Helper to determine category from unit symbol
fn get_unit_category(unit: &str) -> Option<ContextCategory> {
    let u = unit.trim().to_lowercase();
    match u.as_str() {
        // Length
        "mm" | "millimeter" | "millimeters" |
        "cm" | "centimeter" | "centimeters" |
        "m" | "meter" | "meters" |
        "km" | "kilometer" | "kilometers" |
        "in" | "inch" | "inches" |
        "ft" | "foot" | "feet" |
        "yd" | "yard" | "yards" |
        "mi" | "mile" | "miles" => Some(ContextCategory::Length),
        
        // Mass
        "mg" | "milligram" | "milligrams" |
        "g" | "gram" | "grams" |
        "kg" | "kilogram" | "kilograms" |
        "oz" | "ounce" | "ounces" |
        "lb" | "lbs" | "pound" | "pounds" => Some(ContextCategory::Mass),
        
        // Volume
        "ml" | "milliliter" | "milliliters" |
        "l" | "liter" | "liters" | "litre" | "litres" |
        "fl-oz" | "floz" | "fluid ounce" | "fluid ounces" |
        "cup" | "cups" |
        "gal" | "gallon" | "gallons" => Some(ContextCategory::Volume),
        
        // Temperature
        "c" | "celsius" | "f" | "fahrenheit" | "k" | "kelvin" => Some(ContextCategory::Temperature),
        
        // Speed
        "km/h" | "kmh" | "mph" | "kph" => Some(ContextCategory::Speed),
        
        _ => None,
    }
}

/// Get category for widget type
pub fn get_widget_category(widget_type: &str) -> Option<ContextCategory> {
    match widget_type {
        "translator" => Some(ContextCategory::Text),
        "definition" => Some(ContextCategory::Text),
        "text_analyser" => Some(ContextCategory::Text),
        "currency" => Some(ContextCategory::Currency),
        "time_converter" => Some(ContextCategory::Time),
        "unit_converter" => None, // Unit converter handles multiple categories
        "clipboard" => None, // Clipboard is general
        "settings" => None, // Settings is general
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_length() {
        assert_eq!(detect_content_category("12km"), Some(ContextCategory::Length));
        assert_eq!(detect_content_category("100 meters"), Some(ContextCategory::Length));
        assert_eq!(detect_content_category("5.5 ft"), Some(ContextCategory::Length));
    }

    #[test]
    fn test_detect_mass() {
        assert_eq!(detect_content_category("50kg"), Some(ContextCategory::Mass));
        assert_eq!(detect_content_category("10 pounds"), Some(ContextCategory::Mass));
    }

    #[test]
    fn test_detect_currency() {
        assert_eq!(detect_content_category("$100"), Some(ContextCategory::Currency));
        assert_eq!(detect_content_category("50 EUR"), Some(ContextCategory::Currency));
    }

    #[test]
    fn test_detect_temperature() {
        assert_eq!(detect_content_category("25C"), Some(ContextCategory::Temperature));
        assert_eq!(detect_content_category("72 fahrenheit"), Some(ContextCategory::Temperature));
    }

    #[test]
    fn test_get_action_category_currency() {
        use crate::shared::types::CurrencyPayload;
        assert_eq!(
            get_action_category(&ActionType::ConvertCurrency(CurrencyPayload { target_currency: "USD".to_string() })), 
            Some(ContextCategory::Currency)
        );
        assert_eq!(
            get_action_category(&ActionType::ConvertCurrency(CurrencyPayload { target_currency: "EUR".to_string() })), 
            Some(ContextCategory::Currency)
        );
    }

    // #[test]
    // fn test_get_action_category_length() {
    //     // TODO: Re-enable once ConvertUnit allows category resolution (currently returns None)
    //     // assert_eq!(get_action_category(&ActionType::ConvertToKM), Some(ContextCategory::Length));
    // }

    #[test]
    fn test_get_action_category_text() {
        use crate::shared::types::TranslatePayload;
        assert_eq!(
            get_action_category(&ActionType::Translate(TranslatePayload { 
                target_lang: "en".to_string(), 
                source_lang: None
            })), 
            Some(ContextCategory::Text)
        );
    }
}

