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
        // 26 translation language actions
        vec![
            CommandItem { id: "translate_en".to_string(), label: "Translate to English".to_string(), description: Some("Translate selected text to English".to_string()), action_type: Some(ActionType::TranslateEn), widget_type: None, category: None },
            CommandItem { id: "translate_zh".to_string(), label: "Translate to Chinese".to_string(), description: Some("Translate selected text to Chinese".to_string()), action_type: Some(ActionType::TranslateZh), widget_type: None, category: None },
            CommandItem { id: "translate_es".to_string(), label: "Translate to Spanish".to_string(), description: Some("Translate selected text to Spanish".to_string()), action_type: Some(ActionType::TranslateEs), widget_type: None, category: None },
            CommandItem { id: "translate_fr".to_string(), label: "Translate to French".to_string(), description: Some("Translate selected text to French".to_string()), action_type: Some(ActionType::TranslateFr), widget_type: None, category: None },
            CommandItem { id: "translate_de".to_string(), label: "Translate to German".to_string(), description: Some("Translate selected text to German".to_string()), action_type: Some(ActionType::TranslateDe), widget_type: None, category: None },
            CommandItem { id: "translate_ar".to_string(), label: "Translate to Arabic".to_string(), description: Some("Translate selected text to Arabic".to_string()), action_type: Some(ActionType::TranslateAr), widget_type: None, category: None },
            CommandItem { id: "translate_pt".to_string(), label: "Translate to Portuguese".to_string(), description: Some("Translate selected text to Portuguese".to_string()), action_type: Some(ActionType::TranslatePt), widget_type: None, category: None },
            CommandItem { id: "translate_ru".to_string(), label: "Translate to Russian".to_string(), description: Some("Translate selected text to Russian".to_string()), action_type: Some(ActionType::TranslateRu), widget_type: None, category: None },
            CommandItem { id: "translate_ja".to_string(), label: "Translate to Japanese".to_string(), description: Some("Translate selected text to Japanese".to_string()), action_type: Some(ActionType::TranslateJa), widget_type: None, category: None },
            CommandItem { id: "translate_hi".to_string(), label: "Translate to Hindi".to_string(), description: Some("Translate selected text to Hindi".to_string()), action_type: Some(ActionType::TranslateHi), widget_type: None, category: None },
            CommandItem { id: "translate_it".to_string(), label: "Translate to Italian".to_string(), description: Some("Translate selected text to Italian".to_string()), action_type: Some(ActionType::TranslateIt), widget_type: None, category: None },
            CommandItem { id: "translate_nl".to_string(), label: "Translate to Dutch".to_string(), description: Some("Translate selected text to Dutch".to_string()), action_type: Some(ActionType::TranslateNl), widget_type: None, category: None },
            CommandItem { id: "translate_pl".to_string(), label: "Translate to Polish".to_string(), description: Some("Translate selected text to Polish".to_string()), action_type: Some(ActionType::TranslatePl), widget_type: None, category: None },
            CommandItem { id: "translate_tr".to_string(), label: "Translate to Turkish".to_string(), description: Some("Translate selected text to Turkish".to_string()), action_type: Some(ActionType::TranslateTr), widget_type: None, category: None },
            CommandItem { id: "translate_hy".to_string(), label: "Translate to Armenian".to_string(), description: Some("Translate selected text to Armenian".to_string()), action_type: Some(ActionType::TranslateHy), widget_type: None, category: None },
            CommandItem { id: "translate_fa".to_string(), label: "Translate to Persian".to_string(), description: Some("Translate selected text to Persian".to_string()), action_type: Some(ActionType::TranslateFa), widget_type: None, category: None },
            CommandItem { id: "translate_vi".to_string(), label: "Translate to Vietnamese".to_string(), description: Some("Translate selected text to Vietnamese".to_string()), action_type: Some(ActionType::TranslateVi), widget_type: None, category: None },
            CommandItem { id: "translate_id".to_string(), label: "Translate to Indonesian".to_string(), description: Some("Translate selected text to Indonesian".to_string()), action_type: Some(ActionType::TranslateId), widget_type: None, category: None },
            CommandItem { id: "translate_ko".to_string(), label: "Translate to Korean".to_string(), description: Some("Translate selected text to Korean".to_string()), action_type: Some(ActionType::TranslateKo), widget_type: None, category: None },
            CommandItem { id: "translate_bn".to_string(), label: "Translate to Bengali".to_string(), description: Some("Translate selected text to Bengali".to_string()), action_type: Some(ActionType::TranslateBn), widget_type: None, category: None },
            CommandItem { id: "translate_ur".to_string(), label: "Translate to Urdu".to_string(), description: Some("Translate selected text to Urdu".to_string()), action_type: Some(ActionType::TranslateUr), widget_type: None, category: None },
            CommandItem { id: "translate_th".to_string(), label: "Translate to Thai".to_string(), description: Some("Translate selected text to Thai".to_string()), action_type: Some(ActionType::TranslateTh), widget_type: None, category: None },
            CommandItem { id: "translate_sv".to_string(), label: "Translate to Swedish".to_string(), description: Some("Translate selected text to Swedish".to_string()), action_type: Some(ActionType::TranslateSv), widget_type: None, category: None },
            CommandItem { id: "translate_da".to_string(), label: "Translate to Danish".to_string(), description: Some("Translate selected text to Danish".to_string()), action_type: Some(ActionType::TranslateDa), widget_type: None, category: None },
            CommandItem { id: "translate_fi".to_string(), label: "Translate to Finnish".to_string(), description: Some("Translate selected text to Finnish".to_string()), action_type: Some(ActionType::TranslateFi), widget_type: None, category: None },
            CommandItem { id: "translate_hu".to_string(), label: "Translate to Hungarian".to_string(), description: Some("Translate selected text to Hungarian".to_string()), action_type: Some(ActionType::TranslateHu), widget_type: None, category: None },
        ]
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
        let target_lang = match action {
            ActionType::TranslateEn => "en",
            ActionType::TranslateZh => "zh",
            ActionType::TranslateEs => "es",
            ActionType::TranslateFr => "fr",
            ActionType::TranslateDe => "de",
            ActionType::TranslateAr => "ar",
            ActionType::TranslatePt => "pt",
            ActionType::TranslateRu => "ru",
            ActionType::TranslateJa => "ja",
            ActionType::TranslateHi => "hi",
            ActionType::TranslateIt => "it",
            ActionType::TranslateNl => "nl",
            ActionType::TranslatePl => "pl",
            ActionType::TranslateTr => "tr",
            ActionType::TranslateHy => "hy",
            ActionType::TranslateFa => "fa",
            ActionType::TranslateVi => "vi",
            ActionType::TranslateId => "id",
            ActionType::TranslateKo => "ko",
            ActionType::TranslateBn => "bn",
            ActionType::TranslateUr => "ur",
            ActionType::TranslateTh => "th",
            ActionType::TranslateSv => "sv",
            ActionType::TranslateDa => "da",
            ActionType::TranslateFi => "fi",
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
#[tauri::command]
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