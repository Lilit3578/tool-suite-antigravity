//! Translator feature
//!
//! Provides translation functionality with 26 language support.

use crate::types::*;
use crate::settings::AppSettings;
use crate::context;
use super::Feature;
use std::collections::HashMap;

pub struct TranslatorFeature;

impl Feature for TranslatorFeature {
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
        }]
    }
    
    fn action_commands(&self) -> Vec<CommandItem> {
        let translations = vec![
            ("translate_en", "Translate to English", ActionType::TranslateEn),
            ("translate_zh", "Translate to Chinese (Mandarin)", ActionType::TranslateZh),
            ("translate_es", "Translate to Spanish", ActionType::TranslateEs),
            ("translate_fr", "Translate to French", ActionType::TranslateFr),
            ("translate_de", "Translate to German", ActionType::TranslateDe),
            ("translate_ar", "Translate to Arabic", ActionType::TranslateAr),
            ("translate_pt", "Translate to Portuguese", ActionType::TranslatePt),
            ("translate_ru", "Translate to Russian", ActionType::TranslateRu),
            ("translate_ja", "Translate to Japanese", ActionType::TranslateJa),
            ("translate_hi", "Translate to Hindi", ActionType::TranslateHi),
            ("translate_it", "Translate to Italian", ActionType::TranslateIt),
            ("translate_nl", "Translate to Dutch", ActionType::TranslateNl),
            ("translate_pl", "Translate to Polish", ActionType::TranslatePl),
            ("translate_tr", "Translate to Turkish", ActionType::TranslateTr),
            ("translate_hy", "Translate to Armenian", ActionType::TranslateHy),
            ("translate_fa", "Translate to Persian", ActionType::TranslateFa),
            ("translate_vi", "Translate to Vietnamese", ActionType::TranslateVi),
            ("translate_id", "Translate to Indonesian", ActionType::TranslateId),
            ("translate_ko", "Translate to Korean", ActionType::TranslateKo),
            ("translate_bn", "Translate to Bengali", ActionType::TranslateBn),
            ("translate_ur", "Translate to Urdu", ActionType::TranslateUr),
            ("translate_th", "Translate to Thai", ActionType::TranslateTh),
            ("translate_sv", "Translate to Swedish", ActionType::TranslateSv),
            ("translate_da", "Translate to Danish", ActionType::TranslateDa),
            ("translate_fi", "Translate to Finnish", ActionType::TranslateFi),
            ("translate_hu", "Translate to Hungarian", ActionType::TranslateHu),
        ];
        
        translations
            .into_iter()
            .map(|(id, label, action_type)| CommandItem {
                id: id.to_string(),
                label: label.to_string(),
                description: None,
                action_type: Some(action_type),
                widget_type: None,
            })
            .collect()
    }
    
    fn execute_action(
        &self,
        action: &ActionType,
        params: &serde_json::Value,
    ) -> Result<ExecuteActionResponse, String> {
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
            _ => return Err("Not a translation action".to_string()),
        };
        
        let text = params.get("text")
            .and_then(|v| v.as_str())
            .ok_or("Missing 'text' parameter")?;
        let source_lang = params.get("source_lang")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        
        let translate_request = TranslateRequest {
            text: text.to_string(),
            source_lang,
            target_lang: target_lang.to_string(),
            provider: None,
        };
        
        // Execute translation synchronously using tokio block_on
        let response = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(translate_text(translate_request))
        })?;
        
        Ok(ExecuteActionResponse {
            result: response.translated,
            metadata: Some(serde_json::json!({
                "detected_lang": response.detected_source_lang,
            })),
        })
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

/// Translate text between languages
///
/// Uses the unofficial Google Translate API endpoint (free tier).
/// For production, consider using the official Google Cloud Translation API.
#[tauri::command]
pub async fn translate_text(request: TranslateRequest) -> Result<TranslateResponse, String> {
    let _settings = AppSettings::load().unwrap_or_default();
    
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
                            Ok(TranslateResponse {
                                translated: format!("[Translation of: {}]", request.text),
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
                        Ok(TranslateResponse {
                            translated: format!("[Translation of: {}]", request.text),
                            detected_source_lang: Some("auto".to_string()),
                        })
                    }
                }
            } else {
                eprintln!("Translation API returned error: {}", response.status());
                Ok(TranslateResponse {
                    translated: format!("[Translation of: {}]", request.text),
                    detected_source_lang: Some("auto".to_string()),
                })
            }
        }
        Err(e) => {
            eprintln!("Translation API request failed: {}", e);
            Ok(TranslateResponse {
                translated: format!("[Translation of: {}]", request.text),
                detected_source_lang: Some("auto".to_string()),
            })
        }
    }
}