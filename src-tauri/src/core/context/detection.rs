use regex::Regex;
use serde::{Deserialize, Serialize};

/// Context information extracted from selected text
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextInfo {
    pub text: String,
    pub detected_currency: Option<CurrencyInfo>,
    pub detected_language: Option<String>,
    pub source_app: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurrencyInfo {
    pub amount: f64,
    pub currency_code: String,
}

/// Detect currency in text using regex patterns
pub fn detect_currency(text: &str) -> Option<CurrencyInfo> {
    // Truncate input to prevent DoS attacks from large text
    const MAX_INPUT_LENGTH: usize = 1000;
    let truncated_text = if text.len() > MAX_INPUT_LENGTH {
        &text[..MAX_INPUT_LENGTH]
    } else {
        text
    };
    
    // Fuzzy pattern: optional prefix, number, optional whitespace, optional suffix
    static CURRENCY_REGEX: std::sync::OnceLock<Regex> = std::sync::OnceLock::new();
    let re = CURRENCY_REGEX.get_or_init(|| {
        Regex::new(r"(?i)^\s*([^\d\s\.,]*)[\s]*([\d\.,]+)[\s]*([^\d\s\.,]*)\s*$")
            .expect("valid currency regex")
    });

    if let Some(caps) = re.captures(truncated_text.trim()) {
        let prefix = caps.get(1).map(|m| m.as_str()).unwrap_or("").trim();
        let number_raw = caps.get(2).map(|m| m.as_str()).unwrap_or("");
        let suffix = caps.get(3).map(|m| m.as_str()).unwrap_or("").trim();

        let cleaned = number_raw.replace(',', "");
        if let Ok(amount) = cleaned.parse::<f64>() {
            // Detect currency from prefix/suffix tokens
            let map_token = |raw: &str| -> Option<&'static str> {
                let t = raw.trim().to_ascii_lowercase();
                if t.is_empty() {
                    return None;
                }
                match t.as_str() {
                    "$" | "usd" | "dollar" | "dollars" => Some("USD"),
                    "€" | "eur" | "euro" | "euros" => Some("EUR"),
                    "£" | "gbp" | "pound" | "pounds" => Some("GBP"),
                    "¥" | "jpy" | "yen" => Some("JPY"),
                    "cad" => Some("CAD"),
                    "aud" => Some("AUD"),
                    "chf" => Some("CHF"),
                    "cny" => Some("CNY"),
                    "inr" => Some("INR"),
                    "mxn" => Some("MXN"),
                    _ => None,
                }
            };

            let currency_code = map_token(prefix)
                .map(|c| c.to_string())
                .or_else(|| map_token(suffix).map(|c| c.to_string()))
                .or_else(|| {
                    let sp = prefix.to_ascii_uppercase();
                    if sp.len() == 3 && sp.chars().all(|c| c.is_ascii_alphabetic()) {
                        Some(sp)
                    } else {
                        let ss = suffix.to_ascii_uppercase();
                        if ss.len() == 3 && ss.chars().all(|c| c.is_ascii_alphabetic()) {
                            Some(ss)
                        } else {
                            None
                        }
                    }
                });

            if let Some(code) = currency_code {
                return Some(CurrencyInfo { amount, currency_code: code });
            }
        }
    }

    None
}

/// Detect language from text
/// This is a simple heuristic - for production, use the translation API's detection
pub fn detect_language(text: &str) -> Option<String> {
    // Simple heuristic based on character sets
    let has_chinese = text.chars().any(|c| {
        ('\u{4E00}'..='\u{9FFF}').contains(&c) || // CJK Unified Ideographs
        ('\u{3400}'..='\u{4DBF}').contains(&c)    // CJK Extension A
    });
    
    let has_japanese = text.chars().any(|c| {
        ('\u{3040}'..='\u{309F}').contains(&c) || // Hiragana
        ('\u{30A0}'..='\u{30FF}').contains(&c)    // Katakana
    });
    
    let has_korean = text.chars().any(|c| {
        ('\u{AC00}'..='\u{D7AF}').contains(&c)    // Hangul Syllables
    });
    
    let has_arabic = text.chars().any(|c| {
        ('\u{0600}'..='\u{06FF}').contains(&c) || // Arabic
        ('\u{0750}'..='\u{077F}').contains(&c)    // Arabic Supplement
    });
    
    let has_cyrillic = text.chars().any(|c| {
        ('\u{0400}'..='\u{04FF}').contains(&c)    // Cyrillic
    });

    if has_chinese && !has_japanese {
        Some("zh".to_string())
    } else if has_japanese {
        Some("ja".to_string())
    } else if has_korean {
        Some("ko".to_string())
    } else if has_arabic {
        Some("ar".to_string())
    } else if has_cyrillic {
        Some("ru".to_string())
    } else {
        // Default to English for Latin script
        Some("en".to_string())
    }
}

/// Analyze text and extract context information
pub fn analyze_context(text: &str, source_app: Option<String>) -> ContextInfo {
    ContextInfo {
        text: text.to_string(),
        detected_currency: detect_currency(text),
        detected_language: detect_language(text),
        source_app,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_currency_usd_symbol() {
        // Input must match regex logic (essentially clean currency string)
        let result = detect_currency("$123.45");
        assert!(result.is_some());
        if let Some(info) = result {
            assert_eq!(info.currency_code, "USD");
            assert!((info.amount - 123.45).abs() < 0.01);
        }
    }

    #[test]
    fn test_detect_currency_eur_symbol() {
        // Input must match regex logic
        let result = detect_currency("€50.00");
        assert!(result.is_some());
        if let Some(info) = result {
            assert_eq!(info.currency_code, "EUR");
            assert!((info.amount - 50.0).abs() < 0.01);
        }
    }

    #[test]
    fn test_detect_currency_code() {
        let result = detect_currency("Total: 1000 JPY");
        assert!(result.is_some());
        if let Some(info) = result {
            assert_eq!(info.currency_code, "JPY");
            assert!((info.amount - 1000.0).abs() < 0.01);
        }
    }

    #[test]
    fn test_detect_language_chinese() {
        let result = detect_language("你好世界");
        assert_eq!(result, Some("zh".to_string()));
    }

    #[test]
    fn test_detect_language_japanese() {
        let result = detect_language("こんにちは");
        assert_eq!(result, Some("ja".to_string()));
    }

    #[test]
    fn test_detect_language_english() {
        let result = detect_language("Hello world");
        assert_eq!(result, Some("en".to_string()));
    }
}
