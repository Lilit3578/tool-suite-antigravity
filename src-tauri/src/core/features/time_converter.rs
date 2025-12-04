// Time Zone Converter - Logic Layer
// All time calculations happen here in Rust using chrono and chrono-tz

pub mod constants;

use chrono::{DateTime, Local, TimeZone, Utc, Offset};
use chrono_tz::Tz;
use chrono_english::{parse_date_string, Dialect};
use crate::shared::types::{ConvertTimeRequest, ConvertTimeResponse, TimezoneInfo, ParsedTimeInput, CommandItem, ActionType, ExecuteActionResponse};
use super::Feature;
use std::collections::HashMap;
use regex::Regex;

/// Timezone abbreviation to IANA ID mappings
const TIMEZONE_ABBREVIATIONS: &[(&str, &str)] = &[
    ("EST", "America/New_York"),
    ("EDT", "America/New_York"),
    ("PST", "America/Los_Angeles"),
    ("PDT", "America/Los_Angeles"),
    ("CST", "America/Chicago"),  // US Central (default for ambiguous CST)
    ("CDT", "America/Chicago"),
    ("MST", "America/Denver"),
    ("MDT", "America/Denver"),
    ("GMT", "Europe/London"),
    ("UTC", "UTC"),
    ("CET", "Europe/Paris"),
    ("CEST", "Europe/Paris"),
    ("JST", "Asia/Tokyo"),
    ("KST", "Asia/Seoul"),
    ("IST", "Asia/Kolkata"),
    ("AEST", "Australia/Sydney"),
    ("AEDT", "Australia/Sydney"),
    ("NZST", "Pacific/Auckland"),
    ("NZDT", "Pacific/Auckland"),
];

pub struct TimeConverterFeature;

impl Feature for TimeConverterFeature {
    fn id(&self) -> &str {
        "time_converter"
    }
    
    fn widget_commands(&self) -> Vec<CommandItem> {
        vec![CommandItem {
            id: "widget_time_converter".to_string(),
            label: "Time Zone Converter".to_string(),
            description: Some("Open time zone converter widget".to_string()),
            action_type: None,
            widget_type: Some("time_converter".to_string()),
            category: None, // Will be assigned in get_all_command_items()
        }]
    }
    
    fn action_commands(&self) -> Vec<CommandItem> {
        generate_timezone_commands()
    }
    
    fn execute_action(
        &self,
        action: &ActionType,
        params: &serde_json::Value,
    ) -> Result<ExecuteActionResponse, String> {
        match action {
            ActionType::ConvertTime(target_timezone) => {
                let time_input = params.get("text")
                    .and_then(|v| v.as_str())
                    .unwrap_or("now");
                
                let source_timezone = params.get("source_timezone")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());
                
                let request = ConvertTimeRequest {
                    time_input: time_input.to_string(),
                    target_timezone: target_timezone.clone(),
                    source_timezone,
                };
                
                let response = parse_and_convert_time(request)?;
                
                Ok(ExecuteActionResponse {
                    result: format!("{} → {}", response.source_time, response.target_time),
                    metadata: Some(serde_json::json!({
                        "offset_description": response.offset_description,
                        "source_timezone": response.source_timezone,
                        "target_timezone": response.target_timezone,
                    })),
                })
            }
            _ => Err("Not a time conversion action".to_string()),
        }
    }
    
    fn get_context_boost(&self, _captured_text: &str) -> HashMap<String, f64> {
        HashMap::new()
    }
}

/// Parse natural language time input and convert to target timezone
pub fn parse_and_convert_time(request: ConvertTimeRequest) -> Result<ConvertTimeResponse, String> {
    // Determine source timezone (default to Local if not specified)
    let source_tz_str = request.source_timezone.as_deref().unwrap_or("Local");
    
    // Parse time input using chrono-english
    let now = Local::now();
    let parsed_dt = parse_date_string(&request.time_input, now, Dialect::Us)
        .map_err(|e| format!("Failed to parse time input '{}': {}", request.time_input, e))?;
    
    // Convert parsed DateTime to source timezone
    let source_dt = if source_tz_str == "Local" {
        parsed_dt
    } else {
        let source_tz: Tz = source_tz_str.parse()
            .map_err(|_| format!("Invalid source timezone: {}", source_tz_str))?;
        
        // Convert Local to UTC, then to source timezone
        let utc_dt = parsed_dt.with_timezone(&Utc);
        source_tz.from_utc_datetime(&utc_dt.naive_utc()).with_timezone(&Local)
    };
    
    // Parse target timezone
    let target_tz: Tz = request.target_timezone.parse()
        .map_err(|_| format!("Invalid target timezone: {}", request.target_timezone))?;
    
    // Convert to target timezone
    let target_dt = source_dt.with_timezone(&target_tz);
    
    // Calculate UTC offsets
    let source_offset_seconds = source_dt.offset().fix().local_minus_utc();
    let target_offset_seconds = target_dt.offset().fix().local_minus_utc();
    
    let source_utc_offset = format_utc_offset(source_offset_seconds);
    let target_utc_offset = format_utc_offset(target_offset_seconds);
    
    // Get DST-aware zone abbreviations
    let source_zone_abbr = source_dt.format("%Z").to_string();
    let target_zone_abbr = target_dt.format("%Z").to_string();
    
    // Calculate relative offset
    let diff_seconds = (target_offset_seconds - source_offset_seconds).abs();
    let hours = diff_seconds / 3600;
    let minutes = (diff_seconds % 3600) / 60;
    let sign = if target_offset_seconds > source_offset_seconds { "+" } else { "-" };
    let relative_offset = if diff_seconds == 0 {
        "Same time".to_string()
    } else {
        format!("{}{}h {}m", sign, hours, minutes)
    };
    
    // Detect date change
    let source_date = source_dt.date_naive();
    let target_date = target_dt.date_naive();
    let date_change_indicator = if target_date > source_date {
        Some("Next day".to_string())
    } else if target_date < source_date {
        Some("Previous day".to_string())
    } else {
        None
    };
    
    // Calculate offset description (kept for compatibility)
    let offset_hours = target_offset_seconds as f64 / 3600.0;
    let source_offset_hours = source_offset_seconds as f64 / 3600.0;
    let diff_hours = offset_hours - source_offset_hours;
    
    let offset_description = if diff_hours > 0.0 {
        format!("{:.1} hours ahead", diff_hours)
    } else if diff_hours < 0.0 {
        format!("{:.1} hours behind", diff_hours.abs())
    } else {
        "Same time".to_string()
    };
    
    // Format: 11:30pm, 04 dec (lowercase am/pm, short month like currency converter)
    let source_formatted = source_dt.format("%I:%M%P, %d %b").to_string();
    let target_formatted = target_dt.format("%I:%M%P, %d %b").to_string();
    
    Ok(ConvertTimeResponse {
        source_time: source_formatted,
        target_time: target_formatted,
        offset_description,
        source_timezone: source_tz_str.to_string(),
        target_timezone: request.target_timezone,
        target_utc_offset,
        target_zone_abbr,
        relative_offset,
        date_change_indicator,
        source_zone_abbr,
        source_utc_offset,
    })
}

/// Format UTC offset as "UTC±HH:MM"
fn format_utc_offset(offset_seconds: i32) -> String {
    let hours = offset_seconds / 3600;
    let minutes = (offset_seconds % 3600).abs() / 60;
    format!("UTC{:+03}:{:02}", hours, minutes)
}

/// Get all available timezones with city name labels
pub fn get_all_timezones() -> Vec<TimezoneInfo> {
    constants::ALL_TIMEZONES
        .iter()
        .map(|(label, iana_id, keywords)| {
            // Extract city name from IANA ID (e.g., "Asia/Tokyo" -> "Tokyo")
            let city_name = iana_id
                .split('/')
                .last()
                .unwrap_or(label)
                .replace('_', " ");
            
            TimezoneInfo {
                label: city_name,
                iana_id: iana_id.to_string(),
                keywords: keywords.to_string(),
            }
        })
        .collect()
}

/// Generate dynamic command items for all timezones
pub fn generate_timezone_commands() -> Vec<CommandItem> {
    constants::ALL_TIMEZONES
        .iter()
        .map(|(label, iana_id, _keywords)| {
            CommandItem {
                id: format!("time_{}", iana_id.replace('/', "_").to_lowercase()),
                label: format!("Time in {}", label),
                description: Some(format!("Convert time to {} ({})", label, iana_id)),
                action_type: Some(ActionType::ConvertTime(iana_id.to_string())),
                widget_type: None,
                category: None, // Will be assigned in get_all_command_items()
            }
        })
        .collect()
}

/// Detect timezone from text using multiple strategies
fn detect_timezone_from_text(text: &str) -> Option<String> {
    let text_lower = text.to_lowercase();
    
    // Strategy 1: Check for IANA timezone IDs (e.g., "Asia/Tokyo", "America/New_York")
    // Match pattern: word/word
    let iana_regex = Regex::new(r"\b([A-Z][a-z]+/[A-Z][a-z_]+)\b").ok()?;
    if let Some(caps) = iana_regex.captures(text) {
        if let Some(iana_match) = caps.get(1) {
            let iana_id = iana_match.as_str();
            // Verify it's a valid timezone by checking if it exists in our constants
            if constants::ALL_TIMEZONES.iter().any(|(_, id, _)| id == &iana_id) {
                return Some(iana_id.to_string());
            }
        }
    }
    
    // Strategy 2: Check for timezone abbreviations (EST, PST, etc.)
    for (abbr, iana_id) in TIMEZONE_ABBREVIATIONS {
        let abbr_pattern = format!(r"\b{}\b", abbr.to_lowercase());
        if Regex::new(&abbr_pattern).ok()?.is_match(&text_lower) {
            return Some(iana_id.to_string());
        }
    }
    
    // Strategy 3: Check for city/country names in timezone labels
    // Match patterns like "in Tokyo", "Tokyo time", or just "Tokyo"
    for (label, iana_id, keywords) in constants::ALL_TIMEZONES {
        let label_lower = label.to_lowercase();
        let keywords_lower = keywords.to_lowercase();
        
        // Check if label appears in text
        if text_lower.contains(&label_lower) {
            return Some(iana_id.to_string());
        }
        
        // Check if any keyword appears in text
        for keyword in keywords_lower.split_whitespace() {
            if text_lower.contains(keyword) && keyword.len() > 3 {  // Avoid short false matches
                return Some(iana_id.to_string());
            }
        }
    }
    
    // No timezone detected
    None
}

/// Extract time portion from text by removing detected timezone keywords
fn extract_time_portion(text: &str, detected_timezone: &Option<String>) -> String {
    let mut cleaned_text = text.to_string();
    
    if let Some(tz) = detected_timezone {
        // Remove the IANA ID if present
        cleaned_text = cleaned_text.replace(tz, "");
        
        // Remove timezone abbreviations
        for (abbr, _) in TIMEZONE_ABBREVIATIONS {
            let abbr_pattern = Regex::new(&format!(r"\b{}\b", abbr)).ok();
            if let Some(regex) = abbr_pattern {
                cleaned_text = regex.replace_all(&cleaned_text, "").to_string();
            }
        }
        
        // Remove common timezone-related phrases
        let phrases_to_remove = vec![
            r"\bin\s+[A-Za-z]+",  // "in Tokyo"
            r"[A-Za-z]+\s+time",  // "Tokyo time"
        ];
        
        for pattern in phrases_to_remove {
            if let Ok(regex) = Regex::new(pattern) {
                cleaned_text = regex.replace_all(&cleaned_text, "").to_string();
            }
        }
    }
    
    // Clean up extra whitespace
    cleaned_text = cleaned_text.trim().to_string();
    cleaned_text = Regex::new(r"\s+").ok()
        .map(|r| r.replace_all(&cleaned_text, " ").to_string())
        .unwrap_or(cleaned_text);
    
    // If we cleaned everything away, default to "now"
    if cleaned_text.is_empty() {
        "now".to_string()
    } else {
        cleaned_text
    }
}

/// Parse time from selected text, extracting both time and timezone
pub fn parse_time_from_text(text: &str) -> ParsedTimeInput {
    let detected_timezone = detect_timezone_from_text(text);
    let time_input = extract_time_portion(text, &detected_timezone);
    
    ParsedTimeInput {
        time_input,
        source_timezone: detected_timezone,
    }
}

/// Tauri command to parse time from selection
#[tauri::command]
pub async fn parse_time_from_selection(text: String) -> Result<ParsedTimeInput, String> {
    Ok(parse_time_from_text(&text))
}

/// Tauri command to convert time
#[tauri::command]
pub async fn convert_time(request: ConvertTimeRequest) -> Result<ConvertTimeResponse, String> {
    parse_and_convert_time(request)
}

/// Tauri command to get all timezones
#[tauri::command]
pub async fn get_timezones() -> Result<Vec<TimezoneInfo>, String> {
    Ok(get_all_timezones())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_and_convert_time() {
        let request = ConvertTimeRequest {
            time_input: "now".to_string(),
            target_timezone: "Asia/Tokyo".to_string(),
            source_timezone: None,
        };
        
        let result = parse_and_convert_time(request);
        assert!(result.is_ok());
        
        let response = result.unwrap();
        assert!(!response.source_time.is_empty());
        assert!(!response.target_time.is_empty());
        assert!(!response.offset_description.is_empty());
    }

    #[test]
    fn test_get_all_timezones() {
        let timezones = get_all_timezones();
        assert_eq!(timezones.len(), 190);
        assert_eq!(timezones[0].label, "Afghanistan");
        assert_eq!(timezones[0].iana_id, "Asia/Kabul");
    }

    #[test]
    fn test_generate_timezone_commands() {
        let commands = generate_timezone_commands();
        assert_eq!(commands.len(), 190);
        assert!(commands[0].label.starts_with("Time in"));
    }
}
