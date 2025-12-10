use std::sync::OnceLock;
use regex::Regex;
use crate::shared::types::ParsedTimeInput;
use super::constants;

/// Static Regex Patterns using OnceLock for performance
static IANA_REGEX: OnceLock<Regex> = OnceLock::new();
static FORMATTED_DATE_REGEX: OnceLock<Regex> = OnceLock::new();
static TIME_FORMAT_REGEX: OnceLock<Regex> = OnceLock::new();
static WHITESPACE_REGEX: OnceLock<Regex> = OnceLock::new();

/// Initialize generic regex patterns
fn get_iana_regex() -> &'static Regex {
    IANA_REGEX.get_or_init(|| Regex::new(r"(?i)\b([A-Za-z]+/[A-Za-z_]+)\b").unwrap())
}

fn get_formatted_date_regex() -> &'static Regex {
    FORMATTED_DATE_REGEX.get_or_init(|| Regex::new(r"(?i)\d{1,2}\s+(jan|feb|mar|apr|may|jun|jul|aug|sep|oct|nov|dec)").unwrap())
}

fn get_time_format_regex() -> &'static Regex {
    TIME_FORMAT_REGEX.get_or_init(|| Regex::new(r"(?i)\d{1,2}:\d{2}\s*(am|pm)").unwrap())
}

fn get_whitespace_regex() -> &'static Regex {
    WHITESPACE_REGEX.get_or_init(|| Regex::new(r"\s+").unwrap())
}

/// Detect if text is a conversion result to avoid re-parsing
fn is_conversion_result(text: &str) -> bool {
    // Pattern 1: Contains " - " separator with timezone format
    let has_dash_separator = text.contains(" - ");
    let has_timezone_format = text.contains('/') && text.contains('(') && text.contains(')');
    
    // Pattern 2: Contains formatted date like "04 dec" or "04 Dec"
    let has_formatted_date = get_formatted_date_regex().is_match(text);
    
    // Pattern 3: Contains time with am/pm and formatted date
    let has_time_format = get_time_format_regex().is_match(text);
    
    // If it has the dash separator AND either timezone format OR (time format AND date format)
    has_dash_separator && (has_timezone_format || (has_time_format && has_formatted_date))
}

/// Detect timezone from text
/// Returns: Option<(iana_id, matched_keyword)>
fn detect_timezone_from_text(text: &str) -> Option<(String, Option<String>)> {
    let text_lower = text.to_lowercase();
    
    if is_conversion_result(text) {
        return None;
    }
    
    // Strategy 1: Check for IANA timezone IDs
    if let Some(caps) = get_iana_regex().captures(text) {
        if let Some(iana_match) = caps.get(1) {
            let iana_id = iana_match.as_str();
            for (_, id, _) in constants::ALL_TIMEZONES {
                if id.eq_ignore_ascii_case(iana_id) {
                    return Some((id.to_string(), None));
                }
            }
        }
    }
    
    // Strategy 2: Check for timezone abbreviations using static regexes map
    // Note: Since abbreviations are finite, we can iterate, but caching regexes for them 
    // might be overkill if we blindly create them. However, for a fixed set, we could.
    // Given the previous implementation recompiled them every time, let's just use string matching 
    // for exact word boundaries if possible, or compile on the fly but less frequently.
    // Better optimization: Use a single regex for all abbreviations like `\b(EST|PST|...)\b`
    // but that loses the mapping. Let's stick to the map iteration but optimize the check.
    
    // Strategy 2: Check for timezone abbreviations using static map
    // We prioritize specific IANA IDs for common abbreviations (e.g., EST -> America/New_York)
    
    let preferred_timezones = [
        "America/New_York", // EST/EDT
        "America/Los_Angeles", // PST/PDT
        "America/Chicago", // CST/CDT
        "America/London", // GMT/BST
        "Europe/London",
        "Europe/Paris", // CET
        "Asia/Tokyo", // JST
    ];

    let mut candidates = Vec::new();

    for (label, abbr) in super::constants::TIMEZONE_ABBREVIATIONS_MAP {
       // Check for whole word match without Regex overhead
       if has_whole_word(&text_lower, &abbr.to_lowercase()) {
             // Abbreviation matched! Find IANA ID.
             if let Some((_, iana_id, _)) = constants::ALL_TIMEZONES.iter().find(|(l, _, _)| l == label) {
                 candidates.push((iana_id.to_string(), Some(abbr.to_string())));
             }
       }
    }
    
    // Sort candidates to prioritize preferred timezones
    if !candidates.is_empty() {
        candidates.sort_by(|(id_a, _), (id_b, _)| {
            let rank_a = preferred_timezones.iter().position(|&p| p == id_a).unwrap_or(999);
            let rank_b = preferred_timezones.iter().position(|&p| p == id_b).unwrap_or(999);
            rank_a.cmp(&rank_b)
        });
        return Some(candidates[0].clone());
    }
    
    // Strategy 3: Check for city/country names
    for (label, iana_id, keywords) in constants::ALL_TIMEZONES {
        let label_lower = label.to_lowercase();
        
        if text_lower.contains(&label_lower) {
            return Some((iana_id.to_string(), Some(label.to_string())));
        }
        
        // Optimize keyword search
        for keyword in keywords.split_whitespace() {
            let kw_lower = keyword.to_lowercase();
            
            if kw_lower.len() > 3 {
                 if text_lower.contains(&kw_lower) {
                     // Fast path for long keywords
                     return Some((iana_id.to_string(), Some(keyword.to_string()))); 
                 }
            } else {
                 // Strict match for short keywords
                 if has_whole_word(&text_lower, &kw_lower) {
                    return Some((iana_id.to_string(), Some(keyword.to_string())));
                 }
            }
        }
    }
    
    None
}

/// Helper: Check if `word` exists in `text` as a whole word (surrounded by boundaries)
/// Case-insensitive checking should be done by caller passing lowercase strings.
fn has_whole_word(text: &str, word: &str) -> bool {
    let text_len = text.len();
    let word_len = word.len();
    
    if word_len == 0 || word_len > text_len {
        return false;
    }
    
    for (idx, _) in text.match_indices(word) {
        // Check character before
        let boundary_start = if idx == 0 {
            true
        } else {
            let char_before = text[..idx].chars().last();
             match char_before {
                Some(c) => !c.is_alphanumeric(),
                None => true,
            }
        };
        
        // Check character after
        let boundary_end = if idx + word_len >= text_len {
            true
        } else {
            let char_after = text[idx+word_len..].chars().next();
            match char_after {
                Some(c) => !c.is_alphanumeric(),
                None => true,
            }
        };
        
        if boundary_start && boundary_end {
            return true;
        }
    }
    false
}

fn extract_time_portion(text: &str, detected_timezone: &Option<String>) -> String {
    let mut cleaned_text = text.to_string();
    
    if let Some(tz) = detected_timezone {
        // Remove the IANA ID
        let tz_regex = Regex::new(&format!(r"(?i)\b{}\b", regex::escape(tz))).ok();
        if let Some(re) = tz_regex {
             cleaned_text = re.replace_all(&cleaned_text, "").to_string();
        }

        // Remove timezone abbreviations (brute force removal of known abbrs for this tz)
        for (abbr, _) in super::constants::TIMEZONE_ABBREVIATIONS_MAP {
             if let Ok(re) = Regex::new(&format!(r"(?i)\b{}\b", abbr)) {
                 cleaned_text = re.replace_all(&cleaned_text, "").to_string();
             }
        }
        
        // Remove the detected timezone's label and keywords
        for (label, iana_id, keywords) in constants::ALL_TIMEZONES {
            if iana_id == tz {
                 if let Ok(re) = Regex::new(&format!(r"(?i)\b{}\b", regex::escape(label))) {
                     cleaned_text = re.replace_all(&cleaned_text, "").to_string();
                 }
                
                for keyword in keywords.split_whitespace() {
                    // Clean ALL keywords associated with this timezone, regardless of length
                    // Use word boundaries to be safe
                     if let Ok(re) = Regex::new(&format!(r"(?i)\b{}\b", regex::escape(keyword))) {
                         cleaned_text = re.replace_all(&cleaned_text, "").to_string();
                     }
                }
                break;
            }
        }
        
        // Remove common phrases
        let phrases = vec![r"(?i)\bin\s+\w+", r"(?i)\w+\s+time"];
        for pattern in phrases {
             if let Ok(re) = Regex::new(pattern) {
                 cleaned_text = re.replace_all(&cleaned_text, "").to_string();
             }
        }
    }
    
    cleaned_text = cleaned_text.trim().to_string();
    cleaned_text = get_whitespace_regex().replace_all(&cleaned_text, " ").to_string();
    
    if cleaned_text.is_empty() {
        "now".to_string()
    } else {
        cleaned_text
    }
}

/// Parse time from selected text, extracting both time and timezone
pub fn parse_time_from_text(text: &str) -> Option<ParsedTimeInput> {
    
    // Step 1: Check if this is a conversion result
    if is_conversion_result(text) {
        return None;
    }
    
    // Step 2: Detect timezone from text
    let detected_result = detect_timezone_from_text(text);
    let (detected_timezone, matched_keyword) = match detected_result {
        Some((tz, kw)) => (Some(tz), kw),
        None => (None, None),
    };
    
    // Step 3: Extract time portion (remove timezone info)
    let time_input = extract_time_portion(text, &detected_timezone);
    
    Some(ParsedTimeInput {
        time_input,
        source_timezone: detected_timezone,
        matched_keyword,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_conversion_result() {
        assert!(is_conversion_result("11:30pm, 04 dec - Rome/Italy (CET)"));
        assert!(!is_conversion_result("5pm rome"));
    }

    #[test]
    fn test_detect_timezone() {
        assert_eq!(
            detect_timezone_from_text("5pm EST"),
            Some(("America/New_York".to_string(), Some("EST".to_string())))
        );
         assert_eq!(
            detect_timezone_from_text("5pm est"),
            Some(("America/New_York".to_string(), Some("EST".to_string())))
        );
        // Regression test for short keywords matching
        assert_eq!(
            detect_timezone_from_text("12pm uk"),
            Some(("Europe/London".to_string(), Some("uk".to_string())))
        );
        assert_eq!(
            detect_timezone_from_text("12pm UK"),
            Some(("Europe/London".to_string(), Some("uk".to_string())))
        );
    }
    
    #[test]
    fn test_extract_time_portion() {
        let london = Some("Europe/London".to_string());
        assert_eq!(extract_time_portion("12pm uk", &london), "12pm");
        
        // Ensure partial matches of short keywords are NOT removed if boundaries don't match
        // e.g. "jukebox" contains "uk" but shouldn't match "uk" keyword logic if we were strictly enforcing it
        // However, extract logic iterates all keywords.
        // Let's just test basic extraction for now.
    }
}
