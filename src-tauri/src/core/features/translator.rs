//! Translator feature
//!
//! Provides translation functionality with 26 language support.

use crate::shared::types::*;
use crate::shared::settings::AppSettings;
use crate::core::context;
use super::{FeatureSync, FeatureAsync};
use std::collections::HashMap;
use async_trait::async_trait;

#[derive(Clone)]
pub struct TranslatorFeature;

impl FeatureSync for TranslatorFeature {
    fn id(&self) -> &str {
        "translator"
    }
    
    fn widget_commands(&self) -> Vec<CommandItem> {
        vec![CommandItem {
            id: "widget_translator".to_string(),
            label: "Translator".to_string(),
            description: Some("Open translation widget".to_string()),
            action_type: None,
            widget_type: Some("translator".to_string()),
            category: None, // Will be assigned in get_all_command_items()
        }]
    }
    
    
    fn action_commands(&self) -> Vec<CommandItem> {
        // Phase 1: Generate commands dynamically from language list
        // This replaces 26 hardcoded CommandItem definitions
        const LANGUAGES: &[(&str, &str)] = &[
            ("en", "English"),
            ("zh", "Chinese"),
            ("es", "Spanish"),
            ("fr", "French"),
            ("de", "German"),
            ("ar", "Arabic"),
            ("pt", "Portuguese"),
            ("ru", "Russian"),
            ("ja", "Japanese"),
            ("hi", "Hindi"),
            ("it", "Italian"),
            ("nl", "Dutch"),
            ("pl", "Polish"),
            ("tr", "Turkish"),
            ("hy", "Armenian"),
            ("fa", "Persian"),
            ("vi", "Vietnamese"),
            ("id", "Indonesian"),
            ("ko", "Korean"),
            ("bn", "Bengali"),
            ("ur", "Urdu"),
            ("th", "Thai"),
            ("sv", "Swedish"),
            ("da", "Danish"),
            ("fi", "Finnish"),
            ("hu", "Hungarian"),
        ];
        
        LANGUAGES.iter().map(|(code, name)| CommandItem {
            id: format!("translate_{}", code),
            label: format!("Translate to {}", name),
            description: None,
            action_type: Some(ActionType::Translate(TranslatePayload {
                target_lang: code.to_string(),
                source_lang: None, // Auto-detect source language
            })),
            widget_type: None,
            category: None, // Will be assigned by get_action_category
        }).collect()
    }
    
    fn get_context_boost(&self, captured_text: &str) -> HashMap<String, f64> {
        let mut boost_map = HashMap::new();
        
        // Detect language in text
        if let Some(lang_code) = context::detect_language(captured_text) {
            println!("[Context] Detected language: {}", lang_code);
            
            // Boost translator widget
            boost_map.insert("widget_translator".to_string(), 100.0);
            
            // Boost translation actions (especially if not English)
            if lang_code != "en" {
                boost_map.insert("translate_en".to_string(), 90.0);
            }
            
            // Boost all translation actions moderately
            for id in &[
                "translate_en", "translate_zh", "translate_es", "translate_fr",
                "translate_de", "translate_ar", "translate_pt", "translate_ru",
                "translate_ja", "translate_hi", "translate_it", "translate_nl",
                "translate_pl", "translate_tr", "translate_hy", "translate_fa",
                "translate_vi", "translate_id", "translate_ko", "translate_bn",
                "translate_ur", "translate_th", "translate_sv", "translate_da",
                "translate_fi", "translate_hu",
            ] {
                boost_map.insert(id.to_string(), 60.0);
            }
        }
        
        boost_map
    }
}

#[async_trait]
impl FeatureAsync for TranslatorFeature {
    async fn execute_action(
        &self,
        action: &ActionType,
        params: &serde_json::Value,
    ) -> crate::shared::error::AppResult<ExecuteActionResponse> {
        // Phase 1: Handle BOTH new and old variants for backward compatibility
        let target_lang = match action {
            // NEW: Structured payload variant
            ActionType::Translate(payload) => payload.target_lang.as_str(),
            
            // OLD: Deprecated variants (Phase 4 will remove these)
            #[allow(deprecated)]
            ActionType::TranslateEn => "en",
            #[allow(deprecated)]
            ActionType::TranslateZh => "zh",
            #[allow(deprecated)]
            ActionType::TranslateEs => "es",
            #[allow(deprecated)]
            ActionType::TranslateFr => "fr",
            #[allow(deprecated)]
            ActionType::TranslateDe => "de",
            #[allow(deprecated)]
            ActionType::TranslateAr => "ar",
            #[allow(deprecated)]
            ActionType::TranslatePt => "pt",
            #[allow(deprecated)]
            ActionType::TranslateRu => "ru",
            #[allow(deprecated)]
            ActionType::TranslateJa => "ja",
            #[allow(deprecated)]
            ActionType::TranslateHi => "hi",
            #[allow(deprecated)]
            ActionType::TranslateIt => "it",
            #[allow(deprecated)]
            ActionType::TranslateNl => "nl",
            #[allow(deprecated)]
            ActionType::TranslatePl => "pl",
            #[allow(deprecated)]
            ActionType::TranslateTr => "tr",
            #[allow(deprecated)]
            ActionType::TranslateHy => "hy",
            #[allow(deprecated)]
            ActionType::TranslateFa => "fa",
            #[allow(deprecated)]
            ActionType::TranslateVi => "vi",
            #[allow(deprecated)]
            ActionType::TranslateId => "id",
            #[allow(deprecated)]
            ActionType::TranslateKo => "ko",
            #[allow(deprecated)]
            ActionType::TranslateBn => "bn",
            #[allow(deprecated)]
            ActionType::TranslateUr => "ur",
            #[allow(deprecated)]
            ActionType::TranslateTh => "th",
            #[allow(deprecated)]
            ActionType::TranslateSv => "sv",
            #[allow(deprecated)]
            ActionType::TranslateDa => "da",
            #[allow(deprecated)]
            ActionType::TranslateFi => "fi",
            #[allow(deprecated)]
            ActionType::TranslateHu => "hu",
            _ => return Err(crate::shared::error::AppError::Unknown(crate::shared::errors::ERR_UNSUPPORTED_ACTION.to_string())),
        };
        
        let text = params.get("text")
            .and_then(|v| v.as_str())
            .ok_or_else(|| crate::shared::error::AppError::Validation("Missing 'text' parameter".to_string()))?;
            
        let source_lang = params.get("source_lang")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        
        let translate_request = TranslateRequest {
            text: text.to_string(),
            source_lang,
            target_lang: target_lang.to_string(),
            provider: None,
        };
        
        // Execute translation asynchronously
        let response = translate_text(translate_request).await?;
        
        Ok(ExecuteActionResponse {
            result: response.translated,
            metadata: Some(serde_json::json!({
                "detected_lang": response.detected_source_lang,
            })),
        })
    }
}

/// Translate text between languages
///
/// Uses the unofficial Google Translate API endpoint (free tier).
/// For production, consider using the official Google Cloud Translation API.
pub async fn translate_text(request: TranslateRequest) -> crate::shared::error::AppResult<TranslateResponse> {
    let _settings = AppSettings::load().await.unwrap_or_default();
    
    let client = reqwest::Client::new();
    
    // Determine source and target languages
    let source_lang = request.source_lang.unwrap_or_else(|| "auto".to_string());
    let target_lang = &request.target_lang;
    
    // Build the Google Translate URL
    let url = format!(
        "https://translate.googleapis.com/translate_a/single?client=gtx&sl={}&tl={}&dt=t&q={}",
        source_lang,
        target_lang,
        urlencoding::encode(&request.text)
    );

    // Make API request
    match client.get(&url)
        .header("User-Agent", "Mozilla/5.0")
        .send()
        .await
    {
        Ok(response) => {
            if response.status().is_success() {
                match response.json::<serde_json::Value>().await {
                    Ok(json) => {
                        let mut translated = String::new();
                        
                        if let Some(translations) = json.get(0).and_then(|v| v.as_array()) {
                            for translation in translations {
                                if let Some(text) = translation.get(0).and_then(|v| v.as_str()) {
                                    translated.push_str(text);
                                }
                            }
                        }
                        
                        let detected_lang = json.get(2)
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string());

                        if translated.is_empty() {
                            // Fallback if parsing fails but request succeeded
                            Ok(TranslateResponse {
                                translated: request.text.clone(),
                                detected_source_lang: Some("auto".to_string()),
                            })
                        } else {
                            Ok(TranslateResponse {
                                translated,
                                detected_source_lang: detected_lang,
                            })
                        }
                    }
                    Err(e) => {
                        eprintln!("Failed to parse translation response: {}", e);
                         Err(crate::shared::error::AppError::Unknown(format!("Failed to parse translation API response: {}", e)))
                    }
                }
            } else {
                eprintln!("Translation API returned error: {}", response.status());
                Err(crate::shared::error::AppError::Network(format!("Translation API error: {}", response.status())))
            }
        }
        Err(e) => {
            eprintln!("Translation API request failed: {}", e);
            Err(crate::shared::error::AppError::Network(format!("Translation API request failed: {}", e)))
        }
    }
}