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
        println!("[TimeConverter] 🎬 ========== ACTION EXECUTION ==========");
        println!("[TimeConverter] 🎯 Action: {:?}", action);
        println!("[TimeConverter] 📋 Params: {}", params);
        
        match action {
            ActionType::ConvertTime(target_timezone) => {
                let time_input = params.get("text")
                    .and_then(|v| v.as_str())
                    .unwrap_or("now");
                
                let source_timezone = params.get("source_timezone")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());
                
                println!("[TimeConverter] 📥 Action params parsed:");
                println!("  time_input: '{}'", time_input);
                println!("  target_timezone: '{}'", target_timezone);
                println!("  source_timezone: {:?}", source_timezone);
                
                let request = ConvertTimeRequest {
                    time_input: time_input.to_string(),
                    target_timezone: target_timezone.clone(),
                    source_timezone,
                };
                
                let response = parse_and_convert_time(request)?;
                
                // Only show the target time result (TO field format), not FROM → TO
                // Format: "11:30pm, 04 dec - Rome/Italy (CET)"
                let formatted_result = format!("{} - {}", response.target_time, response.target_timezone);
                
                println!("[TimeConverter] 📤 Action result: '{}'", formatted_result);
                println!("[TimeConverter] ✅ ========== ACTION COMPLETE ==========");
                
                Ok(ExecuteActionResponse {
                    result: formatted_result,
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
    println!("[TimeConverter] 🔵 ========== CONVERSION REQUEST ==========");
    println!("[TimeConverter] 📥 Request: time_input='{}', target_timezone='{}', source_timezone={:?}", 
        request.time_input, request.target_timezone, request.source_timezone);
    
    // Determine source timezone (default to Local if not specified)
    let source_tz_str = request.source_timezone.as_deref().unwrap_or("Local");
    println!("[TimeConverter] 🌍 Source timezone: '{}'", source_tz_str);
    println!("[TimeConverter] 🎯 Target timezone: '{}'", request.target_timezone);
    
    // Parse time input using chrono-english (this gives us a DateTime<Local>)
    let now = Local::now();
    println!("[TimeConverter] ⏰ Current local time: {}", now.format("%Y-%m-%d %H:%M:%S %Z"));
    
    let parsed_local_dt = parse_date_string(&request.time_input, now, Dialect::Us)
        .map_err(|e| format!("Failed to parse time input '{}': {}", request.time_input, e))?;
    println!("[TimeConverter] 📅 Parsed local datetime: {}", parsed_local_dt.format("%Y-%m-%d %H:%M:%S %Z"));
    
    // Interpret the parsed time AS BEING IN the source timezone
    let source_dt: DateTime<Tz> = if source_tz_str == "Local" {
        println!("[TimeConverter] 🏠 Source is Local timezone");
        // If source is Local, we need to get the system's actual timezone
        // The parsed_local_dt is already in the system's local timezone
        let naive = parsed_local_dt.naive_local();
        
        // Get the system's timezone offset at this moment
        let local_offset = parsed_local_dt.offset().fix();
        let offset_seconds = local_offset.local_minus_utc();
        println!("[TimeConverter] 📍 Local offset: {} seconds ({} hours)", offset_seconds, offset_seconds / 3600);
        
        // Convert to UTC first
        let utc_dt = naive - chrono::Duration::seconds(offset_seconds as i64);
        
        // Convert UTC DateTime to Tz type
        let utc_tz_dt = Utc.from_utc_datetime(&utc_dt);
        let result = utc_tz_dt.with_timezone(&chrono_tz::Tz::UTC);
        println!("[TimeConverter] 🌐 Source datetime (UTC): {}", result.format("%Y-%m-%d %H:%M:%S %Z"));
        result
    } else {
        println!("[TimeConverter] 🌍 Parsing source timezone: {}", source_tz_str);
        // Parse the source timezone
        let source_tz: Tz = source_tz_str.parse()
            .map_err(|_| format!("Invalid source timezone: {}", source_tz_str))?;
        
        // Get the naive datetime from the parsed local time
        let naive = parsed_local_dt.naive_local();
        println!("[TimeConverter] 📅 Naive datetime: {}", naive.format("%Y-%m-%d %H:%M:%S"));
        
        // Interpret this naive datetime AS BEING IN the source timezone
        let result = source_tz.from_local_datetime(&naive)
            .single()
            .ok_or_else(|| format!("Ambiguous or invalid time in timezone {}", source_tz_str))?;
        println!("[TimeConverter] 🌐 Source datetime ({}): {}", source_tz_str, result.format("%Y-%m-%d %H:%M:%S %Z"));
        result
    };
    
    // Parse target timezone
    let target_tz: Tz = request.target_timezone.parse()
        .map_err(|_| format!("Invalid target timezone: {}", request.target_timezone))?;
    println!("[TimeConverter] 🎯 Parsed target timezone: {}", request.target_timezone);
    
    // Convert to target timezone
    let target_dt = source_dt.with_timezone(&target_tz);
    println!("[TimeConverter] 🌐 Target datetime ({}): {}", request.target_timezone, target_dt.format("%Y-%m-%d %H:%M:%S %Z"));
    
    // Calculate UTC offsets
    let source_offset_seconds = source_dt.offset().fix().local_minus_utc();
    let target_offset_seconds = target_dt.offset().fix().local_minus_utc();
    
    println!("[TimeConverter] 📊 Offset calculation:");
    println!("  Source offset: {} seconds ({} hours)", source_offset_seconds, source_offset_seconds / 3600);
    println!("  Target offset: {} seconds ({} hours)", target_offset_seconds, target_offset_seconds / 3600);
    
    let source_utc_offset = format_utc_offset(source_offset_seconds);
    let target_utc_offset = format_utc_offset(target_offset_seconds);
    println!("  Source UTC offset: {}", source_utc_offset);
    println!("  Target UTC offset: {}", target_utc_offset);
    
    // Get DST-aware zone abbreviations
    let source_zone_abbr = source_dt.format("%Z").to_string();
    let target_zone_abbr = target_dt.format("%Z").to_string();
    println!("  Source zone abbr: {}", source_zone_abbr);
    println!("  Target zone abbr: {}", target_zone_abbr);
    
    // Format timezone labels with DST-aware abbreviations
    let source_label = if source_tz_str == "Local" {
        format!("Local Time ({})", source_zone_abbr)
    } else {
        format_timezone_label_with_abbr(&source_tz_str, &source_zone_abbr)
    };
    let target_label = format_timezone_label_with_abbr(&request.target_timezone, &target_zone_abbr);
    println!("  Source label: {}", source_label);
    println!("  Target label: {}", target_label);
    
    // Calculate relative offset (FROM → TO direction)
    // Formula: Target - Source = Offset
    // If target is ahead (larger offset), we add time (positive)
    // If target is behind (smaller offset), we subtract time (negative)
    let diff_seconds = target_offset_seconds - source_offset_seconds;
    let abs_diff_seconds = diff_seconds.abs();
    let hours = abs_diff_seconds / 3600;
    let minutes = (abs_diff_seconds % 3600) / 60;
    let sign = if diff_seconds >= 0 { "+" } else { "-" };
    
    println!("[TimeConverter] 🧮 Relative offset calculation:");
    println!("  diff_seconds = target_offset ({}) - source_offset ({}) = {}", 
        target_offset_seconds, source_offset_seconds, diff_seconds);
    println!("  abs_diff_seconds = {}", abs_diff_seconds);
    println!("  hours = {}, minutes = {}", hours, minutes);
    println!("  sign = '{}' ({} >= 0)", sign, diff_seconds);
    
    let relative_offset = if diff_seconds == 0 {
        "Same time".to_string()
    } else {
        format!("{}{}h {}m", sign, hours, minutes)
    };
    println!("  relative_offset = '{}'", relative_offset);
    
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
    
    println!("[TimeConverter] 📝 Formatted times:");
    println!("  source_time: '{}'", source_formatted);
    println!("  target_time: '{}'", target_formatted);
    
    let response = ConvertTimeResponse {
        source_time: source_formatted,
        target_time: target_formatted,
        offset_description,
        source_timezone: source_label,
        target_timezone: target_label,
        target_utc_offset,
        target_zone_abbr,
        relative_offset,
        date_change_indicator,
        source_zone_abbr,
        source_utc_offset,
    };
    
    println!("[TimeConverter] ✅ ========== CONVERSION RESPONSE ==========");
    println!("[TimeConverter] 📤 Response: target_time='{}', relative_offset='{}', source_timezone='{}', target_timezone='{}'",
        response.target_time, response.relative_offset, response.source_timezone, response.target_timezone);
    
    Ok(response)
}

/// Format UTC offset as "UTC±HH:MM"
fn format_utc_offset(offset_seconds: i32) -> String {
    let hours = offset_seconds / 3600;
    let minutes = (offset_seconds % 3600).abs() / 60;
    format!("UTC{:+03}:{:02}", hours, minutes)
}

/// Format timezone label as "City/Country (Abbr)"
/// Example: "Europe/Rome" with country "Italy" → "Rome/Italy (CET)"
fn format_timezone_label(iana_id: &str, country_label: &str) -> String {
    // Extract city name from IANA ID (part after last slash)
    let city = iana_id
        .split('/')
        .last()
        .unwrap_or(iana_id)
        .replace('_', " ");
    
    // Capitalize first letter of each word in city
    let city_formatted: String = city
        .split_whitespace()
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ");
    
    // Get abbreviation from static map (for dropdown labels)
    let abbr = constants::get_timezone_abbreviation(iana_id);
    
    format!("{}/{} ({})", city_formatted, country_label, abbr)
}

/// Format timezone label with DST-aware abbreviation
/// Example: "Europe/Rome" with abbr "CET" → "Rome/Italy (CET)"
fn format_timezone_label_with_abbr(iana_id: &str, abbr: &str) -> String {
    // Find the country label from constants
    let country_label = constants::ALL_TIMEZONES
        .iter()
        .find(|(_, id, _)| *id == iana_id)
        .map(|(country, _, _)| *country)
        .unwrap_or("Unknown");
    
    // Extract city name from IANA ID
    let city = iana_id
        .split('/')
        .last()
        .unwrap_or(iana_id)
        .replace('_', " ");
    
    // Capitalize first letter of each word in city
    let city_formatted: String = city
        .split_whitespace()
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ");
    
    format!("{}/{} ({})", city_formatted, country_label, abbr)
}

/// Get all available timezones with proper labels in "City/Country (Abbr)" format
pub fn get_all_timezones() -> Vec<TimezoneInfo> {
    constants::ALL_TIMEZONES
        .iter()
        .map(|(country_label, iana_id, keywords)| {
            let formatted_label = format_timezone_label(iana_id, country_label);
            TimezoneInfo {
                label: formatted_label,
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
        .map(|(country_label, iana_id, _keywords)| {
            let formatted_label = format_timezone_label(iana_id, country_label);
            CommandItem {
                id: format!("time_{}", iana_id.replace('/', "_").to_lowercase()),
                label: format!("Time in {}", formatted_label),
                description: Some(format!("Convert time to {}", formatted_label)),
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
    println!("[TimeConverter] 🧹 extract_time_portion: input='{}', detected_tz={:?}", text, detected_timezone);
    let mut cleaned_text = text.to_string();
    let original_text = cleaned_text.clone();
    
    if let Some(tz) = detected_timezone {
        println!("[TimeConverter] 🧹 Removing timezone: {}", tz);
        // Remove the IANA ID if present
        cleaned_text = cleaned_text.replace(tz, "");
        
        // Remove timezone abbreviations
        for (abbr, _) in TIMEZONE_ABBREVIATIONS {
            let abbr_pattern = Regex::new(&format!(r"\b{}\b", abbr)).ok();
            if let Some(regex) = abbr_pattern {
                cleaned_text = regex.replace_all(&cleaned_text, "").to_string();
            }
        }
        
        // Remove the detected timezone's label and keywords
        for (label, iana_id, keywords) in constants::ALL_TIMEZONES {
            if iana_id == tz {
                // Remove label (case-insensitive)
                let label_pattern = Regex::new(&format!(r"(?i)\b{}\b", regex::escape(label))).ok();
                if let Some(regex) = label_pattern {
                    cleaned_text = regex.replace_all(&cleaned_text, "").to_string();
                }
                
                // Remove each keyword (case-insensitive)
                for keyword in keywords.split_whitespace() {
                    let keyword_pattern = Regex::new(&format!(r"(?i)\b{}\b", regex::escape(keyword))).ok();
                    if let Some(regex) = keyword_pattern {
                        cleaned_text = regex.replace_all(&cleaned_text, "").to_string();
                    }
                }
                break;
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
    
    println!("[TimeConverter] 🧹 After cleaning: '{}' (was '{}')", cleaned_text, original_text);
    
    // If we cleaned everything away, default to "now"
    if cleaned_text.is_empty() {
        println!("[TimeConverter] 🧹 Text was completely cleaned, defaulting to 'now'");
        "now".to_string()
    } else {
        cleaned_text
    }
}

/// Parse time from selected text, extracting both time and timezone
pub fn parse_time_from_text(text: &str) -> ParsedTimeInput {
    println!("[TimeConverter] 🔍 ========== PARSE TIME FROM TEXT ==========");
    println!("[TimeConverter] 📝 Input text: '{}'", text);
    
    // Check if this looks like a conversion result
    // Patterns: "time - timezone", "time, date - timezone", contains formatted timezone labels
    let has_dash_separator = text.contains(" - ");
    let has_timezone_format = text.contains('/') && text.contains('(');
    let has_timezone_id = constants::ALL_TIMEZONES.iter().any(|(_, id, _)| text.contains(id));
    let has_formatted_date = Regex::new(r"\d{1,2}\s+[a-z]{3}")
        .map(|re| re.is_match(&text.to_lowercase()))
        .unwrap_or(false);
    
    let looks_like_result = (has_dash_separator && (has_timezone_format || has_timezone_id)) ||
        (has_timezone_format && has_formatted_date);
    
    println!("[TimeConverter] 🔍 Conversion result check:");
    println!("  has_dash_separator: {}", has_dash_separator);
    println!("  has_timezone_format: {}", has_timezone_format);
    println!("  has_timezone_id: {}", has_timezone_id);
    println!("  has_formatted_date: {}", has_formatted_date);
    println!("  looks_like_result: {}", looks_like_result);
    
    if looks_like_result {
        println!("[TimeConverter] ⚠️  Text looks like a conversion result, skipping parse");
        return ParsedTimeInput {
            time_input: "now".to_string(),
            source_timezone: None,
        };
    }
    
    let detected_timezone = detect_timezone_from_text(text);
    println!("[TimeConverter] 🌍 Detected timezone: {:?}", detected_timezone);
    
    let time_input = extract_time_portion(text, &detected_timezone);
    println!("[TimeConverter] ⏰ Extracted time input: '{}'", time_input);
    println!("[TimeConverter] ✅ ========== PARSE COMPLETE ==========");
    
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

    #[test]
    fn test_parse_time_rome() {
        let parsed = parse_time_from_text("5pm rome");
        assert_eq!(parsed.source_timezone, Some("Europe/Rome".to_string()));
        assert_eq!(parsed.time_input, "5pm");
    }
}
