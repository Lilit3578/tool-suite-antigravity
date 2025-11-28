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
    // Common currency patterns
    let patterns = vec![
        // $100, $1,234.56
        (r"\$\s*([0-9,]+\.?[0-9]*)", "USD"),
        // €100, €1.234,56
        (r"€\s*([0-9.,]+)", "EUR"),
        // £100, £1,234.56
        (r"£\s*([0-9,]+\.?[0-9]*)", "GBP"),
        // ¥100, ¥1,234
        (r"¥\s*([0-9,]+\.?[0-9]*)", "JPY"),
        // 100 USD, 1234.56 USD
        (r"([0-9,]+\.?[0-9]*)\s*USD", "USD"),
        // 100 EUR, 1234.56 EUR
        (r"([0-9,]+\.?[0-9]*)\s*EUR", "EUR"),
        // 100 GBP, 1234.56 GBP
        (r"([0-9,]+\.?[0-9]*)\s*GBP", "GBP"),
        // 100 JPY, 1234 JPY
        (r"([0-9,]+\.?[0-9]*)\s*JPY", "JPY"),
        // 100 CAD, 1234.56 CAD
        (r"([0-9,]+\.?[0-9]*)\s*CAD", "CAD"),
        // 100 AUD, 1234.56 AUD
        (r"([0-9,]+\.?[0-9]*)\s*AUD", "AUD"),
        // 100 CHF, 1234.56 CHF
        (r"([0-9,]+\.?[0-9]*)\s*CHF", "CHF"),
        // 100 CNY, 1234.56 CNY
        (r"([0-9,]+\.?[0-9]*)\s*CNY", "CNY"),
        // 100 INR, 1234.56 INR
        (r"([0-9,]+\.?[0-9]*)\s*INR", "INR"),
    ];

    for (pattern, currency_code) in patterns {
        if let Ok(re) = Regex::new(pattern) {
            if let Some(captures) = re.captures(text) {
                if let Some(amount_str) = captures.get(1) {
                    // Remove commas and parse
                    let cleaned = amount_str.as_str().replace(",", "");
                    if let Ok(amount) = cleaned.parse::<f64>() {
                        return Some(CurrencyInfo {
                            amount,
                            currency_code: currency_code.to_string(),
                        });
                    }
                }
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
        let result = detect_currency("The price is $123.45");
        assert!(result.is_some());
        let info = result.unwrap();
        assert_eq!(info.currency_code, "USD");
        assert!((info.amount - 123.45).abs() < 0.01);
    }

    #[test]
    fn test_detect_currency_eur_symbol() {
        let result = detect_currency("Cost: €50.00");
        assert!(result.is_some());
        let info = result.unwrap();
        assert_eq!(info.currency_code, "EUR");
        assert!((info.amount - 50.0).abs() < 0.01);
    }

    #[test]
    fn test_detect_currency_code() {
        let result = detect_currency("Total: 1000 JPY");
        assert!(result.is_some());
        let info = result.unwrap();
        assert_eq!(info.currency_code, "JPY");
        assert!((info.amount - 1000.0).abs() < 0.01);
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
