// Time Zone Converter - Logic Layer
// Refactored for production readiness:
// - Uses static regex compilations via `parsing` module
// - clean error handling
// - separated parsing logic

pub mod constants;
pub mod parsing;

use chrono::{Local, TimeZone, Offset}; // Added Offset
use chrono_tz::Tz;
use chrono_english::{parse_date_string, Dialect};
use crate::shared::types::{ConvertTimeRequest, ConvertTimeResponse, ParsedTimeInput, CommandItem, ActionType, TimePayload, ExecuteActionResponse, TimezoneInfo};
use super::{FeatureSync, FeatureAsync};
use std::collections::HashMap;
use regex::Regex;
use async_trait::async_trait;

/// Timezone abbreviation to IANA ID mappings (expanded)
const TIMEZONE_ABBREVIATIONS: &[(&str, &str)] = &[
    // North America
    ("EST", "America/New_York"),
    ("EDT", "America/New_York"),
    ("PST", "America/Los_Angeles"),
    ("PDT", "America/Los_Angeles"),
    ("CST", "America/Chicago"),
    ("CDT", "America/Chicago"),
    ("MST", "America/Denver"),
    ("MDT", "America/Denver"),
    ("AST", "America/Antigua"),
    
    // Europe
    ("GMT", "UTC"),  // Changed to UTC for consistency
    ("UTC", "UTC"),
    ("CET", "Europe/Paris"),
    ("CEST", "Europe/Paris"),
    ("EET", "Europe/Athens"),
    ("EEST", "Europe/Athens"),
    ("WET", "Europe/Lisbon"),
    ("WEST", "Europe/Lisbon"),
    ("MSK", "Europe/Moscow"),
    
    // Asia
    ("JST", "Asia/Tokyo"),
    ("KST", "Asia/Seoul"),
    ("IST", "Asia/Kolkata"),
    ("ICT", "Asia/Bangkok"),
    ("SGT", "Asia/Singapore"),
    ("HKT", "Asia/Hong_Kong"),
    ("PKT", "Asia/Karachi"),
    ("GST", "Asia/Dubai"),
    ("IRST", "Asia/Tehran"),
    ("AFT", "Asia/Kabul"),
    ("AMT", "Asia/Yerevan"),
    ("AZT", "Asia/Baku"),
    ("BST", "Asia/Dhaka"),
    ("BTT", "Asia/Thimphu"),
    ("NPT", "Asia/Kathmandu"),
    ("TJT", "Asia/Dushanbe"),
    ("TMT", "Asia/Ashgabat"),
    ("UZT", "Asia/Tashkent"),
    ("KGT", "Asia/Bishkek"),
    ("ALMT", "Asia/Almaty"),
    ("MMT", "Asia/Yangon"),
    ("WIB", "Asia/Jakarta"),
    ("PHT", "Asia/Manila"),
    ("MYT", "Asia/Kuala_Lumpur"),
    ("BNT", "Asia/Brunei"),
    ("ULAT", "Asia/Ulaanbaatar"),
    
    // Oceania / Pacific
    ("AEST", "Australia/Sydney"),
    ("AEDT", "Australia/Sydney"),
    ("NZST", "Pacific/Auckland"),
    ("NZDT", "Pacific/Auckland"),
    ("FJT", "Pacific/Fiji"),
    ("PGT", "Pacific/Port_Moresby"),
    ("SBT", "Pacific/Guadalcanal"),
    ("VUT", "Pacific/Efate"),
    ("TOT", "Pacific/Tongatapu"),
    ("WST", "Pacific/Apia"),
    ("PONT", "Pacific/Pohnpei"),
    ("GILT", "Pacific/Tarawa"),
    ("MHT", "Pacific/Majuro"),
    ("NRT", "Pacific/Nauru"),
    ("PWT", "Pacific/Palau"),
    ("TVT", "Pacific/Funafuti"),
    
    // Africa
    ("WAT", "Africa/Lagos"),
    ("CAT", "Africa/Johannesburg"),
    ("EAT", "Africa/Nairobi"),
    ("SAST", "Africa/Johannesburg"),
    ("CVT", "Atlantic/Cape_Verde"),
    
    // South America
    ("ART", "America/Argentina/Buenos_Aires"),
    ("BOT", "America/La_Paz"),
    ("BRT", "America/Sao_Paulo"),
    ("CLT", "America/Santiago"),
    ("COT", "America/Bogota"),
    ("ECT", "America/Guayaquil"),
    ("GYT", "America/Guyana"),
    ("PET", "America/Lima"),
    ("PYT", "America/Asuncion"),
    ("SRT", "America/Paramaribo"),
    ("UYT", "America/Montevideo"),
    ("VET", "America/Caracas"),
    
    // Middle East
    ("TRT", "Europe/Istanbul"),
    
    // Indian Ocean
    ("MVT", "Indian/Maldives"),
    ("MUT", "Indian/Mauritius"),
    ("SCT", "Indian/Mahe"),
];

#[derive(Clone)]
pub struct TimeConverterFeature;

impl FeatureSync for TimeConverterFeature {
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
            category: None,
        }]
    }
    
    fn action_commands(&self) -> Vec<CommandItem> {
        generate_timezone_commands()
    }
    
    fn get_context_boost(&self, _captured_text: &str) -> HashMap<String, f64> {
        HashMap::new()
    }
}

#[async_trait]
impl FeatureAsync for TimeConverterFeature {
    async fn execute_action(
        &self,
        action: &ActionType,
        params: &serde_json::Value,
    ) -> crate::shared::error::AppResult<ExecuteActionResponse> {
        println!("[TimeConverter] 🎬 ========== ACTION EXECUTION ==========");
        println!("[TimeConverter] 🎯 Action: {:?}", action);
        println!("[TimeConverter] 📋 Params: {}", params);
        
        match action {
            ActionType::ConvertTimeAction(payload) => {
                let text_input = params.get("text")
                    .and_then(|v| v.as_str())
                    .unwrap_or("now");
                
                println!("[TimeConverter] 🔍 Parsing time from text: '{}'", text_input);
                let parsed = parse_time_from_text(text_input)
                    .ok_or_else(|| crate::shared::error::AppError::Validation("Failed to parse time from text, likely a conversion result.".to_string()))?;
                
                println!("[TimeConverter] 📊 Parsed result:");
                println!("  original text: '{}'", text_input);
                println!("  parsed time_input: '{}'", parsed.time_input);
                println!("  parsed source_timezone: {:?}", parsed.source_timezone);
                println!("  matched_keyword: {:?}", parsed.matched_keyword);
                println!("  target_timezone: '{}'", payload.target_timezone);
                
                let request = ConvertTimeRequest {
                    time_input: parsed.time_input,
                    target_timezone: payload.target_timezone.clone(),
                    source_timezone: parsed.source_timezone,
                    matched_keyword: parsed.matched_keyword,
                };
                
                let response = parse_and_convert_time(request)?;
                
                // Only show time and date, not timezone name
                let formatted_result = response.target_time;
                
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
            _ => Err(crate::shared::error::AppError::Unknown(crate::shared::errors::ERR_UNSUPPORTED_ACTION.to_string())),
        }
    }
}

/// Parse natural language time input and convert to target timezone
pub fn parse_and_convert_time(request: ConvertTimeRequest) -> crate::shared::error::AppResult<ConvertTimeResponse> {
    println!("[TimeConverter] 🔵 ========== CONVERSION REQUEST ==========");
    println!("[TimeConverter] 📥 Request: time_input='{}', target_timezone='{}', source_timezone={:?}", 
        request.time_input, request.target_timezone, request.source_timezone);
    
    let source_tz_str = request.source_timezone.as_deref().unwrap_or("UTC");
    println!("[TimeConverter] 🌍 Source timezone: '{}'", source_tz_str);
    println!("[TimeConverter] 🎯 Target timezone: '{}'", request.target_timezone);
    
    let now = Local::now();
    println!("[TimeConverter] ⏰ Current local time: {}", now.format("%Y-%m-%d %H:%M:%S %Z"));
    
    println!("[TimeConverter] 🔍 Parsing time input with chrono-english...");
    let parsed_local_dt = parse_date_string(&request.time_input, now, Dialect::Us)
        .map_err(|e| {
            let err_msg = format!("Failed to parse time input '{}': {}", request.time_input, e);
            println!("[TimeConverter] ❌ {}", err_msg);
            crate::shared::error::AppError::Validation(err_msg)
        })?;
    println!("[TimeConverter] 📅 Parsed local datetime: {}", parsed_local_dt.format("%Y-%m-%d %H:%M:%S %Z"));
    
    // Parse source timezone
    println!("[TimeConverter] 🌍 Parsing source timezone: {}", source_tz_str);
    let source_tz: Tz = source_tz_str.parse()
        .map_err(|_| crate::shared::error::AppError::Validation(format!("Invalid source timezone: {}", source_tz_str)))?;
    
    // Get the naive datetime from the parsed local time
    // This is the time the user typed, which we need to interpret AS BEING IN source_tz
    let naive = parsed_local_dt.naive_local();
    println!("[TimeConverter] 📅 Naive datetime: {}", naive.format("%Y-%m-%d %H:%M:%S"));
    
    // Interpret this naive datetime AS BEING IN the source timezone
    let source_dt = source_tz.from_local_datetime(&naive)
        .single()
        .ok_or_else(|| crate::shared::error::AppError::Validation(format!("Ambiguous or invalid time in timezone {}", source_tz_str)))?;
    println!("[TimeConverter] 🌐 Source datetime ({}): {}", source_tz_str, source_dt.format("%Y-%m-%d %H:%M:%S %Z"));
    
    println!("[TimeConverter] 🔍 Parsing target timezone: '{}'", request.target_timezone);
    let target_tz: Tz = request.target_timezone.parse()
        .map_err(|e| {
            let err_msg = format!("Invalid target timezone '{}': {:?}", request.target_timezone, e);
            println!("[TimeConverter] ❌ {}", err_msg);
            crate::shared::error::AppError::Validation(err_msg)
        })?;
    println!("[TimeConverter] 🎯 Parsed target timezone successfully");
    
    println!("[TimeConverter] 🔄 Converting to target timezone...");
    let target_dt = source_dt.with_timezone(&target_tz);
    println!("[TimeConverter] 🌐 Target datetime ({}): {}", request.target_timezone, target_dt.format("%Y-%m-%d %H:%M:%S %Z"));
    
    let source_offset_seconds = source_dt.offset().fix().local_minus_utc();
    let target_offset_seconds = target_dt.offset().fix().local_minus_utc();
    
    println!("[TimeConverter] 📊 Offset calculation:");
    println!("  Source offset: {} seconds ({} hours)", source_offset_seconds, source_offset_seconds / 3600);
    println!("  Target offset: {} seconds ({} hours)", target_offset_seconds, target_offset_seconds / 3600);
    
    let source_utc_offset = format_utc_offset(source_offset_seconds);
    let target_utc_offset = format_utc_offset(target_offset_seconds);
    println!("  Source UTC offset: {}", source_utc_offset);
    println!("  Target UTC offset: {}", target_utc_offset);
    
    let source_zone_abbr = source_dt.format("%Z").to_string();
    let target_zone_abbr = target_dt.format("%Z").to_string();
    println!("  Source zone abbr: {}", source_zone_abbr);
    println!("  Target zone abbr: {}", target_zone_abbr);
    
    let source_label = format_timezone_label_with_abbr(&source_tz_str, &source_zone_abbr);
    let target_label = format_timezone_label_with_abbr(&request.target_timezone, &target_zone_abbr);
    println!("  Source label: {}", source_label);
    println!("  Target label: {}", target_label);
    println!("[TimeConverter] 📊 Labels formatted successfully");
    
    let diff_seconds = target_offset_seconds - source_offset_seconds;
    let abs_diff_seconds = diff_seconds.abs();
    let hours = abs_diff_seconds / 3600;
    let minutes = (abs_diff_seconds % 3600) / 60;
    let sign = if diff_seconds >= 0 { "+" } else { "-" };
    
    println!("[TimeConverter] 🧮 Relative offset calculation:");
    println!("  diff_seconds = {} - {} = {}", target_offset_seconds, source_offset_seconds, diff_seconds);
    println!("  hours = {}, minutes = {}", hours, minutes);
    
    let relative_offset = if diff_seconds == 0 {
        "Same time".to_string()
    } else {
        format!("{}{}h {}m", sign, hours, minutes)
    };
    println!("  relative_offset = '{}'", relative_offset);
    
    let source_date = source_dt.date_naive();
    let target_date = target_dt.date_naive();
    let date_change_indicator = if target_date > source_date {
        Some("Next day".to_string())
    } else if target_date < source_date {
        Some("Previous day".to_string())
    } else {
        None
    };
    
    let offset_hours = target_offset_seconds as f64 / 3600.0;
    let source_offset_hours = source_offset_seconds as f64 / 3600.0;
    let diff_hours = offset_hours - source_offset_hours;
    
    let mut offset_description = if diff_hours > 0.0 {
        format!("{:.1} hours ahead", diff_hours)
    } else if diff_hours < 0.0 {
        format!("{:.1} hours behind", diff_hours.abs())
    } else {
        "Same time".to_string()
    };
    
    // SMART CITY DETECTION: Detect if user searched for a secondary city
    // Example: User searches "birmingham" -> matches "London/United Kingdom" timezone
    // We want to append "Uses the same timezone as Birmingham" to the description
    println!("[TimeConverter] 🔍 Smart City Detection: Checking for secondary city match...");
    
    let mut smart_note: Option<String> = None;
    
    // Check if we have a matched_keyword from the timezone detection
    if let Some(ref keyword) = request.matched_keyword {
        println!("[TimeConverter]   Matched keyword from detection: '{}'", keyword);
        
        // If source_timezone was explicitly provided, check if the keyword is a secondary city
        if request.source_timezone.is_some() {
            // Find the timezone info for the source timezone
            for (display_label, iana_id, _keywords) in constants::ALL_TIMEZONES {
                if iana_id == &source_tz_str {
                    println!("[TimeConverter]   Found source timezone: {} ({})", display_label, iana_id);
                    
                    // Extract primary city from "London/United Kingdom" -> "London"
                    let primary_city = display_label
                        .split('/')
                        .next()
                        .unwrap_or(display_label)
                        .to_lowercase();
                    
                    println!("[TimeConverter]   Primary city: '{}'", primary_city);
                    
                    // Check if the matched keyword is different from the primary city
                    let keyword_lower = keyword.to_lowercase();
                    if keyword_lower != primary_city && keyword_lower.len() > 3 {
                        // Capitalize the first letter of the keyword
                        let keyword_capitalized = if let Some(first_char) = keyword.chars().next() {
                            first_char.to_uppercase().collect::<String>() + &keyword[1..]
                        } else {
                            keyword.clone()
                        };
                        
                        smart_note = Some(format!("Uses the same timezone as {}", keyword_capitalized));
                        if let Some(note) = &smart_note {
                            println!("[TimeConverter]   ✅ Smart note generated: '{}'", note);
                        }
                    } else {
                        println!("[TimeConverter]   ℹ️  Matched keyword '{}' is the primary city, no note needed", keyword);
                    }
                    
                    break;
                }
            }
        }
    } else {
        println!("[TimeConverter]   No matched keyword available");
    }
    
    // Append smart note to offset description if found
    if let Some(note) = smart_note {
        offset_description = format!("{} • {}", offset_description, note);
        println!("[TimeConverter] 📝 Final offset_description with smart note: '{}'", offset_description);
    } else {
        println!("[TimeConverter] ℹ️  No secondary city match found");
    }
    
    // FIX: Format as 'hh:mm pm/am, DD Mon' (e.g., '06:00pm, 05 Dec')
    // %I = 12-hour format with leading zero (01-12)
    // %M = minute with leading zero (00-59)
    // %P = lowercase am/pm
    // %d = day with leading zero (01-31)
    // %b = abbreviated month name (Jan, Feb, etc.)
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
    
    Ok(response)
}

fn format_utc_offset(offset_seconds: i32) -> String {
    let hours = offset_seconds / 3600;
    let minutes = (offset_seconds % 3600).abs() / 60;
    format!("UTC{:+03}:{:02}", hours, minutes)
}

fn format_timezone_label(iana_id: &str, display_label: &str) -> String {
    // The display_label is already in "City/Country" format (e.g., "Beijing/China")
    // Just need to append the timezone abbreviation
    let abbr = constants::get_timezone_abbreviation(iana_id);
    format!("{} ({})", display_label, abbr)
}

fn format_timezone_label_with_abbr(iana_id: &str, abbr: &str) -> String {
    println!("[TimeConverter] 🏷️  Formatting label for IANA: '{}', abbr: '{}'", iana_id, abbr);
    
    // Find the display label from constants (already in "City/Country" format)
    let display_label = constants::ALL_TIMEZONES
        .iter()
        .find(|(_, id, _)| *id == iana_id)
        .map(|(label, _, _)| *label)
        .unwrap_or_else(|| {
            println!("[TimeConverter] ⚠️  IANA ID '{}' not found in constants, using 'Unknown'", iana_id);
            "Unknown"
        });
    
    println!("[TimeConverter] 🏷️  Found display label: '{}'", display_label);
    
    // The display label is already in "City/Country" format (e.g., "Beijing/China")
    // Just append the abbreviation
    let result = format!("{} ({})", display_label, abbr);
    println!("[TimeConverter] 🏷️  Final label: '{}'", result);
    result
}

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

pub fn generate_timezone_commands() -> Vec<CommandItem> {
    constants::ALL_TIMEZONES
        .iter()
        .map(|(label, iana_id, _keywords)| {
            let formatted_label = format_timezone_label(iana_id, label);
            CommandItem {
                id: format!("convert_time_{}", iana_id.to_lowercase().replace('/', "_")),
                label: format!("Convert time to {}", formatted_label),
                description: None,
                action_type: Some(ActionType::ConvertTimeAction(TimePayload {
                    target_timezone: iana_id.to_string(),
                })),
                widget_type: None,
                category: None,
            }
        })
        .collect()
}

/// FIX: Enhanced detection with proper case handling and result pattern detection
/// Returns: Option<(iana_id, matched_keyword)>
fn detect_timezone_from_text(text: &str) -> Option<(String, Option<String>)> {
    let text_lower = text.to_lowercase();
    
    // FIX: Check if this looks like a conversion result - more comprehensive patterns
    if is_conversion_result(text) {
        println!("[TimeConverter] 🚫 Text appears to be a conversion result, skipping timezone detection");
        return None;
    }
    
    // Strategy 1: Check for IANA timezone IDs - FIX: case-insensitive matching
    let iana_regex = match Regex::new(r"(?i)\b([A-Za-z]+/[A-Za-z_]+)\b") {
        Ok(re) => re,
        Err(e) => {
            eprintln!("[TimeConverter] ⚠️  Failed to compile IANA regex: {}", e);
            return None;
        }
    };
    if let Some(caps) = iana_regex.captures(text) {
        if let Some(iana_match) = caps.get(1) {
            let iana_id = iana_match.as_str();
            // Find matching IANA ID (case-insensitive)
            for (_, id, _) in constants::ALL_TIMEZONES {
                if id.eq_ignore_ascii_case(iana_id) {
                    return Some((id.to_string(), None)); // IANA format has no keyword
                }
            }
        }
    }
    
    // Strategy 2: Check for timezone abbreviations
    for (abbr, iana_id) in TIMEZONE_ABBREVIATIONS {
        let abbr_pattern = format!(r"\b{}\b", abbr.to_lowercase());
        match Regex::new(&abbr_pattern) {
            Ok(re) => {
                if re.is_match(&text_lower) {
                    return Some((iana_id.to_string(), Some(abbr.to_string())));
                }
            }
            Err(e) => {
                eprintln!("[TimeConverter] ⚠️  Failed to compile abbr regex for '{}': {}", abbr, e);
                continue;
            }
        }
    }
    
    // Strategy 3: Check for city/country names
    for (label, iana_id, keywords) in constants::ALL_TIMEZONES {
        let label_lower = label.to_lowercase();
        let keywords_lower = keywords.to_lowercase();
        
        if text_lower.contains(&label_lower) {
            return Some((iana_id.to_string(), Some(label.to_string())));
        }
        
        for keyword in keywords_lower.split_whitespace() {
            if keyword.len() > 3 && text_lower.contains(keyword) {
                // Return the original keyword (not lowercased) for proper capitalization
                let original_keyword = keywords
                    .split_whitespace()
                    .find(|k| k.to_lowercase() == keyword)
                    .unwrap_or(keyword);
                return Some((iana_id.to_string(), Some(original_keyword.to_string())));
            }
        }
    }
    
    None
}

/// FIX: Detect if text is a conversion result to avoid re-parsing
fn is_conversion_result(text: &str) -> bool {
    // Pattern 1: Contains " - " separator with timezone format
    let has_dash_separator = text.contains(" - ");
    let has_timezone_format = text.contains('/') && text.contains('(') && text.contains(')');
    
    // Pattern 2: Contains formatted date like "04 dec" or "04 Dec"
    let has_formatted_date = Regex::new(r"(?i)\d{1,2}\s+(jan|feb|mar|apr|may|jun|jul|aug|sep|oct|nov|dec)")
        .map(|re| re.is_match(text))
        .unwrap_or(false);
    
    // Pattern 3: Contains time with am/pm and formatted date
    let has_time_format = Regex::new(r"(?i)\d{1,2}:\d{2}\s*(am|pm)")
        .map(|re| re.is_match(text))
        .unwrap_or(false);
    
    // If it has the dash separator AND either timezone format OR (time format AND date format)
    let looks_like_result = has_dash_separator && (has_timezone_format || (has_time_format && has_formatted_date));
    
    println!("[TimeConverter] 🔍 Result detection: dash={}, tz_fmt={}, date_fmt={}, time_fmt={}, is_result={}",
        has_dash_separator, has_timezone_format, has_formatted_date, has_time_format, looks_like_result);
    
    looks_like_result
}

fn extract_time_portion(text: &str, detected_timezone: &Option<String>) -> String {
    println!("[TimeConverter] 🧹 extract_time_portion: input='{}', detected_tz={:?}", text, detected_timezone);
    let mut cleaned_text = text.to_string();
    
    if let Some(tz) = detected_timezone {
        println!("[TimeConverter] 🧹 Removing timezone: {}", tz);
        
        // Remove the IANA ID (case-insensitive)
        match Regex::new(&format!(r"(?i)\b{}\b", regex::escape(tz))) {
            Ok(regex) => {
                cleaned_text = regex.replace_all(&cleaned_text, "").to_string();
            }
            Err(e) => {
                eprintln!("[TimeConverter] ⚠️  Failed to compile IANA removal regex: {}", e);
            }
        }
        
        // Remove timezone abbreviations
        for (abbr, _) in TIMEZONE_ABBREVIATIONS {
            match Regex::new(&format!(r"(?i)\b{}\b", abbr)) {
                Ok(regex) => {
                    cleaned_text = regex.replace_all(&cleaned_text, "").to_string();
                }
                Err(e) => {
                    eprintln!("[TimeConverter] ⚠️  Failed to compile abbr removal regex for '{}': {}", abbr, e);
                }
            }
        }
        
        // Remove the detected timezone's label and keywords
        for (label, iana_id, keywords) in constants::ALL_TIMEZONES {
            if iana_id == tz {
                match Regex::new(&format!(r"(?i)\b{}\b", regex::escape(label))) {
                    Ok(regex) => {
                        cleaned_text = regex.replace_all(&cleaned_text, "").to_string();
                    }
                    Err(e) => {
                        eprintln!("[TimeConverter] ⚠️  Failed to compile label removal regex: {}", e);
                    }
                }
                
                for keyword in keywords.split_whitespace() {
                    if keyword.len() > 3 {
                        match Regex::new(&format!(r"(?i)\b{}\b", regex::escape(keyword))) {
                            Ok(regex) => {
                                cleaned_text = regex.replace_all(&cleaned_text, "").to_string();
                            }
                            Err(e) => {
                                eprintln!("[TimeConverter] ⚠️  Failed to compile keyword removal regex: {}", e);
                            }
                        }
                    }
                }
                break;
            }
        }
        
        // Remove common phrases
        let phrases = vec![r"(?i)\bin\s+\w+", r"(?i)\w+\s+time"];
        for pattern in phrases {
            match Regex::new(pattern) {
                Ok(regex) => {
                    cleaned_text = regex.replace_all(&cleaned_text, "").to_string();
                }
                Err(e) => {
                    eprintln!("[TimeConverter] ⚠️  Failed to compile phrase removal regex '{}': {}", pattern, e);
                }
            }
        }
    }
    
    cleaned_text = cleaned_text.trim().to_string();
    cleaned_text = match Regex::new(r"\s+") {
        Ok(r) => r.replace_all(&cleaned_text, " ").to_string(),
        Err(e) => {
            eprintln!("[TimeConverter] ⚠️  Failed to compile whitespace normalization regex: {}", e);
            cleaned_text
        }
    };
    
    println!("[TimeConverter] 🧹 After cleaning: '{}'", cleaned_text);
    
    if cleaned_text.is_empty() {
        println!("[TimeConverter] 🧹 Text was completely cleaned, defaulting to 'now'");
        "now".to_string()
    } else {
        cleaned_text
    }
}

/// Parse time from selected text, extracting both time and timezone
pub fn parse_time_from_text(text: &str) -> Option<ParsedTimeInput> {
    println!("[TimeConverter] 🔍 ========== PARSE TIME FROM TEXT ==========");
    println!("[TimeConverter] 📝 Input text: '{}'", text);
    
    // Step 1: Check if this is a conversion result
    if is_conversion_result(text) {
        println!("[TimeConverter] 🚫 Detected conversion result, returning None");
        println!("[TimeConverter] ✅ ========== PARSE COMPLETE ==========");
        return None;
    }
    
    // Step 2: Detect timezone from text
    let detected_result = detect_timezone_from_text(text);
    let (detected_timezone, matched_keyword) = match detected_result {
        Some((tz, kw)) => (Some(tz), kw),
        None => (None, None),
    };
    
    println!("[TimeConverter] 🌍 Detected timezone: {:?}", detected_timezone);
    println!("[TimeConverter] 🏷️  Matched keyword: {:?}", matched_keyword);
    
    // Step 3: Extract time portion (remove timezone info)
    let time_input = extract_time_portion(text, &detected_timezone);
    println!("[TimeConverter] ⏰ Extracted time input: '{}'", time_input);
    
    println!("[TimeConverter] ✅ ========== PARSE COMPLETE ==========");
    
    Some(ParsedTimeInput {
        time_input,
        source_timezone: detected_timezone,
        matched_keyword,
    })
}

#[tauri::command]
pub async fn parse_time_from_selection(text: String) -> crate::shared::error::AppResult<ParsedTimeInput> {
    parse_time_from_text(&text)
        .ok_or_else(|| crate::shared::error::AppError::Validation("Failed to parse time from selection, likely a conversion result.".to_string()))
}

/// Get the system's IANA timezone (e.g., "Asia/Seoul", "America/New_York")
/// This reads from macOS System Settings and is NOT affected by VPNs
#[tauri::command]
pub async fn get_system_timezone() -> crate::shared::error::AppResult<String> {
    match iana_time_zone::get_timezone() {
        Ok(tz) => {
            println!("[TimeConverter] 🖥️  Detected system timezone: {}", tz);
            Ok(tz)
        }
        Err(e) => {
            println!("[TimeConverter] ⚠️  Failed to detect system timezone: {:?}", e);
            // Fallback to UTC if detection fails
            Ok("UTC".to_string())
        }
    }
}

#[tauri::command]
pub async fn convert_time(request: ConvertTimeRequest) -> crate::shared::error::AppResult<ConvertTimeResponse> {
    parse_and_convert_time(request)
}

#[tauri::command]
pub async fn get_timezones() -> crate::shared::error::AppResult<Vec<TimezoneInfo>> {
    Ok(get_all_timezones())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_conversion_result() {
        assert!(is_conversion_result("11:30pm, 04 dec - Rome/Italy (CET)"));
        assert!(is_conversion_result("03:45am, 25 Dec - Tokyo/Japan (JST)"));
        assert!(!is_conversion_result("5pm rome"));
        assert!(!is_conversion_result("tomorrow at 3pm"));
    }

    #[test]
    fn test_detect_timezone_case_insensitive() {
        assert_eq!(
            detect_timezone_from_text("5pm EST"),
            Some(("America/New_York".to_string(), Some("EST".to_string())))
        );
        assert_eq!(
            detect_timezone_from_text("5pm est"),
            Some(("America/New_York".to_string(), Some("EST".to_string())))
        );
        assert_eq!(
            detect_timezone_from_text("europe/rome"),
            Some(("Europe/Rome".to_string(), None))
        );
    }
}