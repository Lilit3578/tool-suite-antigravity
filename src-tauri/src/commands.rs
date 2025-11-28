use crate::types::*;
use crate::settings::AppSettings;
use crate::clipboard::{ClipboardHistory, ClipboardItem};
use crate::automation;
use crate::context;
use tauri::Manager;
use tauri_plugin_clipboard_manager::ClipboardExt;
use std::collections::HashMap;

#[tauri::command]
pub async fn capture_selection(app: tauri::AppHandle, mode: Option<String>) -> Result<CaptureResult, String> {
    let source = mode.unwrap_or_else(|| "clipboard".to_string());
    
    if source == "selection" {
        // Capture selected text using Cmd+C
        // First get the active app for context (before we potentially lose focus)
        let active_app = automation::get_active_app().ok();
        
        // If we have a palette window open, we need to restore focus to the original app first
        // Try to get the last active app from app state, or use current active app
        let target_app = active_app.clone().unwrap_or_else(|| "Finder".to_string());
        
        // Restore focus to the original app so we can capture the selection
        if let Err(e) = automation::restore_focus(&target_app) {
            eprintln!("[CaptureSelection] Failed to restore focus to {}: {}", target_app, e);
            eprintln!("[CaptureSelection] Continuing anyway - operation might still work");
            // Continue anyway - might still work
        } else {
            println!("[CaptureSelection] Successfully restored focus to {}", target_app);
        }
        
        // Wait a bit for focus to restore
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        
        // Simulate Cmd+C to copy selection to clipboard
        if let Err(e) = automation::simulate_cmd_c() {
            eprintln!("[CaptureSelection] Failed to simulate Cmd+C: {}", e);
            eprintln!("[CaptureSelection] This may indicate missing accessibility permissions");
        } else {
            println!("[CaptureSelection] Successfully simulated Cmd+C");
        }
        
        // Wait a bit for clipboard to update
        tokio::time::sleep(tokio::time::Duration::from_millis(150)).await;
        
        // Read from clipboard
        match app.clipboard().read_text() {
            Ok(text) => {
                println!("[CaptureSelection] Successfully read {} bytes from clipboard", text.len());
                // Restore focus back to our app (palette window if it exists)
                if let Some(palette_window) = app.get_webview_window("palette-window") {
                    if let Err(e) = palette_window.set_focus() {
                        eprintln!("[CaptureSelection] Failed to restore focus to palette window: {}", e);
                    }
                }
                
                Ok(CaptureResult {
                    text,
                    source: active_app.unwrap_or_else(|| "unknown".to_string()),
                })
            }
            Err(e) => {
                eprintln!("[CaptureSelection] Failed to read clipboard after Cmd+C: {}", e);
                eprintln!("[CaptureSelection] Returning empty result");
                // Restore focus back
                if let Some(palette_window) = app.get_webview_window("palette-window") {
                    if let Err(e) = palette_window.set_focus() {
                        eprintln!("[CaptureSelection] Failed to restore focus to palette window: {}", e);
                    }
                }
                Ok(CaptureResult {
                    text: String::new(),
                    source: "selection".to_string(),
                })
            }
        }
    } else {
        // Just read current clipboard without Cmd+C
        match app.clipboard().read_text() {
            Ok(text) => {
                Ok(CaptureResult {
                    text,
                    source: "clipboard".to_string(),
                })
            }
            Err(e) => {
                eprintln!("Failed to read clipboard: {}", e);
                Ok(CaptureResult {
                    text: String::new(),
                    source: "clipboard".to_string(),
                })
            }
        }
    }
}

#[tauri::command]
pub async fn translate_text(request: TranslateRequest) -> Result<TranslateResponse, String> {
    let _settings = AppSettings::load().unwrap_or_default();
    
    // Use Google Translate API (unofficial free endpoint)
    // This uses the public Google Translate service without requiring an API key
    // For production, consider using the official Google Cloud Translation API
    
    let client = reqwest::Client::new();
    
    // Determine source and target languages
    let source_lang = request.source_lang.unwrap_or_else(|| "auto".to_string());
    let target_lang = &request.target_lang;
    
    // Build the Google Translate URL
    // Using the public translate_a/single endpoint
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
                        // Parse Google Translate response format
                        // Response is an array: [[[translated_text, original_text, null, null, score]...], null, source_lang]
                        let mut translated = String::new();
                        
                        if let Some(translations) = json.get(0).and_then(|v| v.as_array()) {
                            for translation in translations {
                                if let Some(text) = translation.get(0).and_then(|v| v.as_str()) {
                                    translated.push_str(text);
                                }
                            }
                        }
                        
                        // Get detected source language
                        let detected_lang = json.get(2)
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string());

                        if translated.is_empty() {
                            // Fallback if parsing failed
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
                        // Fallback to mock response
                        Ok(TranslateResponse {
                            translated: format!("[Translation of: {}]", request.text),
                            detected_source_lang: Some("auto".to_string()),
                        })
                    }
                }
            } else {
                eprintln!("Translation API returned error: {}", response.status());
                // Fallback to mock response
                Ok(TranslateResponse {
                    translated: format!("[Translation of: {}]", request.text),
                    detected_source_lang: Some("auto".to_string()),
                })
            }
        }
        Err(e) => {
            eprintln!("Translation API request failed: {}", e);
            // Fallback to mock response
            Ok(TranslateResponse {
                translated: format!("[Translation of: {}]", request.text),
                detected_source_lang: Some("auto".to_string()),
            })
        }
    }
}

#[tauri::command]
pub async fn convert_currency(request: ConvertCurrencyRequest) -> Result<ConvertCurrencyResponse, String> {
    let settings = AppSettings::load().unwrap_or_default();
    
    // Use exchangerate-api.com (free tier: 1500 requests/month)
    let api_key = if !settings.api_keys.currency_api_key.is_empty() {
        settings.api_keys.currency_api_key.clone()
    } else {
        // Use free tier endpoint (no API key required, but limited)
        String::new()
    };

    let client = reqwest::Client::new();
    
    // Build API URL
    let api_url = if api_key.is_empty() {
        // Free tier endpoint
        format!("https://api.exchangerate-api.com/v4/latest/{}", request.from)
    } else {
        // Paid tier with API key
        format!("https://v6.exchangerate-api.com/v6/{}/latest/{}", api_key, request.from)
    };

    // Make API request
    match client.get(&api_url)
        .send()
        .await
    {
        Ok(response) => {
            if response.status().is_success() {
                match response.json::<serde_json::Value>().await {
                    Ok(json) => {
                        // Extract the exchange rate
                        let rates = &json["rates"];
                        let rate = rates[&request.to]
                            .as_f64()
                            .unwrap_or(1.0);
                        
                        let result = request.amount * rate;
                        let timestamp = json["time_last_updated"]
                            .as_str()
                            .or(json["date"].as_str())
                            .unwrap_or("")
                            .to_string();

                        Ok(ConvertCurrencyResponse {
                            result,
                            rate,
                            timestamp: if timestamp.is_empty() {
                                chrono::Utc::now().to_rfc3339()
                            } else {
                                timestamp
                            },
                        })
                    }
                    Err(e) => {
                        eprintln!("Failed to parse currency response: {}", e);
                        // Fallback to mock rate
                        let rate = 1.07;
                        Ok(ConvertCurrencyResponse {
                            result: request.amount * rate,
                            rate,
                            timestamp: chrono::Utc::now().to_rfc3339(),
                        })
                    }
                }
            } else {
                eprintln!("Currency API returned error: {}", response.status());
                // Fallback to mock rate
                let rate = 1.07;
                Ok(ConvertCurrencyResponse {
                    result: request.amount * rate,
                    rate,
                    timestamp: chrono::Utc::now().to_rfc3339(),
                })
            }
        }
        Err(e) => {
            eprintln!("Currency API request failed: {}", e);
            // Fallback to mock rate
            let rate = 1.07;
            Ok(ConvertCurrencyResponse {
                result: request.amount * rate,
                rate,
                timestamp: chrono::Utc::now().to_rfc3339(),
            })
        }
    }
}

#[tauri::command]
pub async fn get_settings() -> Result<AppSettings, String> {
    AppSettings::load()
}

#[tauri::command]
pub async fn save_settings(settings: AppSettings) -> Result<(), String> {
    settings.save()
}

#[tauri::command]
pub async fn log_message(request: LogRequest) -> Result<(), String> {
    println!("[{}] {}", request.level.to_uppercase(), request.message);
    Ok(())
}

// New commands for unified command palette
#[tauri::command]
pub async fn get_command_items(
    _app: tauri::AppHandle,
    metrics: tauri::State<'_, context::UsageMetrics>,
    captured_text: Option<String>,
) -> Result<Vec<crate::types::CommandItem>, String> {
    use crate::types::{ActionType, CommandItem};
    
    let mut items = vec![];
    
    // Widget commands (4)
    items.push(CommandItem {
        id: "widget_clipboard".to_string(),
        label: "Clipboard History".to_string(),
        description: Some("View and paste from clipboard history".to_string()),
        action_type: None,
        widget_type: Some("clipboard".to_string()),
    });
    items.push(CommandItem {
        id: "widget_translator".to_string(),
        label: "Translator".to_string(),
        description: Some("Open translation widget".to_string()),
        action_type: None,
        widget_type: Some("translator".to_string()),
    });
    items.push(CommandItem {
        id: "widget_currency".to_string(),
        label: "Currency Converter".to_string(),
        description: Some("Open currency converter widget".to_string()),
        action_type: None,
        widget_type: Some("currency".to_string()),
    });
    items.push(CommandItem {
        id: "widget_settings".to_string(),
        label: "Settings".to_string(),
        description: Some("Open settings".to_string()),
        action_type: None,
        widget_type: Some("settings".to_string()),
    });
    
    // Translation actions (26)
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
    
    for (id, label, action_type) in &translations {
        items.push(CommandItem {
            id: id.to_string(),
            label: label.to_string(),
            description: None,
            action_type: Some(action_type.clone()),
            widget_type: None,
        });
    }
    
    // Currency conversion actions (10)
    let conversions = vec![
        ("convert_usd", "Convert to US Dollar (USD)", ActionType::ConvertUsd),
        ("convert_eur", "Convert to Euro (EUR)", ActionType::ConvertEur),
        ("convert_gbp", "Convert to British Pound (GBP)", ActionType::ConvertGbp),
        ("convert_jpy", "Convert to Japanese Yen (JPY)", ActionType::ConvertJpy),
        ("convert_aud", "Convert to Australian Dollar (AUD)", ActionType::ConvertAud),
        ("convert_cad", "Convert to Canadian Dollar (CAD)", ActionType::ConvertCad),
        ("convert_chf", "Convert to Swiss Franc (CHF)", ActionType::ConvertChf),
        ("convert_cny", "Convert to Chinese Yuan (CNY)", ActionType::ConvertCny),
        ("convert_inr", "Convert to Indian Rupee (INR)", ActionType::ConvertInr),
        ("convert_mxn", "Convert to Mexican Peso (MXN)", ActionType::ConvertMxn),
    ];
    
    for (id, label, action_type) in &conversions {
        items.push(CommandItem {
            id: id.to_string(),
            label: label.to_string(),
            description: None,
            action_type: Some(action_type.clone()),
            widget_type: None,
        });
    }
    
    // Apply context-aware ranking if we have captured text
    let mut context_boost: Option<HashMap<String, f64>> = None;
    
    if let Some(text) = captured_text {
        let mut boost_map = HashMap::new();
        
        // Detect currency in text
        if let Some(currency_info) = context::detect_currency(&text) {
            println!("[Context] Detected currency: {} {}", currency_info.amount, currency_info.currency_code);
            
            // Boost currency widget
            boost_map.insert("widget_currency".to_string(), 100.0);
            
            // Boost currency conversion actions
            for (id, _, _) in &conversions {
                boost_map.insert(id.to_string(), 80.0);
            }
        }
        
        // Detect language in text
        if let Some(lang_code) = context::detect_language(&text) {
            println!("[Context] Detected language: {}", lang_code);
            
            // Boost translator widget
            boost_map.insert("widget_translator".to_string(), 100.0);
            
            // Boost translation actions (especially if not English)
            if lang_code != "en" {
                boost_map.insert("translate_en".to_string(), 90.0);
            }
            
            // Boost all translation actions moderately
            for (id, _, _) in &translations {
                boost_map.insert(id.to_string(), 60.0);
            }
        }
        
        if !boost_map.is_empty() {
            context_boost = Some(boost_map);
        }
    }
    
    // Rank commands using usage metrics and context
    let ranked_items = context::rank_commands(
        items,
        |cmd| cmd.id.clone(),
        &metrics,
        context_boost,
    );
    
    Ok(ranked_items)
}


#[tauri::command]
pub async fn execute_action(request: crate::types::ExecuteActionRequest) -> Result<crate::types::ExecuteActionResponse, String> {
    use crate::types::{ActionType, ExecuteActionResponse};
    
    // Helper to extract target language code from action type
    let get_target_lang = |action: &ActionType| -> Option<&str> {
        match action {
            ActionType::TranslateEn => Some("en"),
            ActionType::TranslateZh => Some("zh"),
            ActionType::TranslateEs => Some("es"),
            ActionType::TranslateFr => Some("fr"),
            ActionType::TranslateDe => Some("de"),
            ActionType::TranslateAr => Some("ar"),
            ActionType::TranslatePt => Some("pt"),
            ActionType::TranslateRu => Some("ru"),
            ActionType::TranslateJa => Some("ja"),
            ActionType::TranslateHi => Some("hi"),
            ActionType::TranslateIt => Some("it"),
            ActionType::TranslateNl => Some("nl"),
            ActionType::TranslatePl => Some("pl"),
            ActionType::TranslateTr => Some("tr"),
            ActionType::TranslateHy => Some("hy"),
            ActionType::TranslateFa => Some("fa"),
            ActionType::TranslateVi => Some("vi"),
            ActionType::TranslateId => Some("id"),
            ActionType::TranslateKo => Some("ko"),
            ActionType::TranslateBn => Some("bn"),
            ActionType::TranslateUr => Some("ur"),
            ActionType::TranslateTh => Some("th"),
            ActionType::TranslateSv => Some("sv"),
            ActionType::TranslateDa => Some("da"),
            ActionType::TranslateFi => Some("fi"),
            ActionType::TranslateHu => Some("hu"),
            _ => None,
        }
    };
    
    // Helper to extract target currency code from action type
    let get_target_currency = |action: &ActionType| -> Option<&str> {
        match action {
            ActionType::ConvertUsd => Some("USD"),
            ActionType::ConvertEur => Some("EUR"),
            ActionType::ConvertGbp => Some("GBP"),
            ActionType::ConvertJpy => Some("JPY"),
            ActionType::ConvertAud => Some("AUD"),
            ActionType::ConvertCad => Some("CAD"),
            ActionType::ConvertChf => Some("CHF"),
            ActionType::ConvertCny => Some("CNY"),
            ActionType::ConvertInr => Some("INR"),
            ActionType::ConvertMxn => Some("MXN"),
            _ => None,
        }
    };
    
    // Handle translation actions
    if let Some(target_lang) = get_target_lang(&request.action_type) {
        let text = request.params.get("text")
            .and_then(|v| v.as_str())
            .ok_or("Missing 'text' parameter")?;
        let source_lang = request.params.get("source_lang")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        
        let translate_request = TranslateRequest {
            text: text.to_string(),
            source_lang,
            target_lang: target_lang.to_string(),
            provider: None,
        };
        
        let response = translate_text(translate_request).await?;
        
        return Ok(ExecuteActionResponse {
            result: response.translated,
            metadata: Some(serde_json::json!({
                "detected_lang": response.detected_source_lang,
            })),
        });
    }
    
    // Handle currency conversion actions
    if let Some(target_currency) = get_target_currency(&request.action_type) {
        // Try to get amount and from currency from params
        let text = request.params.get("text")
            .and_then(|v| v.as_str())
            .unwrap_or("100");
        
        // Parse amount from text (simple number extraction)
        let amount = text.chars()
            .filter(|c| c.is_numeric() || *c == '.')
            .collect::<String>()
            .parse::<f64>()
            .unwrap_or(100.0);
        
        // Default from currency (could be improved with detection)
        let from = request.params.get("from")
            .and_then(|v| v.as_str())
            .unwrap_or("USD");
        
        let convert_request = ConvertCurrencyRequest {
            amount,
            from: from.to_string(),
            to: target_currency.to_string(),
            date: None,
        };
        
        let response = convert_currency(convert_request).await?;
        
        return Ok(ExecuteActionResponse {
            result: format!("{:.2} {}", response.result, target_currency),
            metadata: Some(serde_json::json!({
                "rate": response.rate,
                "timestamp": response.timestamp,
            })),
        });
    }
    
    Err("Unknown action type".to_string())
}

// Cursor positioning commands
#[tauri::command]
pub fn get_cursor_position() -> Result<crate::types::CursorPosition, String> {
    use mouse_position::mouse_position::Mouse;
    
    match Mouse::get_mouse_position() {
        Mouse::Position { x, y } => Ok(crate::types::CursorPosition {
            x: x as f64,
            y: y as f64,
        }),
        Mouse::Error => Err("Failed to get cursor position".to_string()),
    }
}

#[tauri::command]
pub fn get_primary_monitor_bounds(app: tauri::AppHandle) -> Result<crate::types::ScreenBounds, String> {
    // Get the primary monitor
    let monitor = app.primary_monitor()
        .map_err(|e| format!("Failed to get primary monitor: {}", e))?
        .ok_or("No primary monitor found")?;
    
    let position = monitor.position();
    let size = monitor.size();
    
    Ok(crate::types::ScreenBounds {
        x: position.x as f64,
        y: position.y as f64,
        width: size.width as f64,
        height: size.height as f64,
    })
}

#[tauri::command]
pub fn calculate_palette_position(
    cursor_x: f64,
    cursor_y: f64,
    screen_x: f64,
    screen_y: f64,
    screen_width: f64,
    screen_height: f64,
) -> crate::types::WindowPosition {
    const PALETTE_WIDTH: f64 = 270.0;
    const PALETTE_HEIGHT: f64 = 328.0;
    const POPOVER_WIDTH: f64 = 280.0; // Space for popover on the right
    const TOTAL_WIDTH: f64 = PALETTE_WIDTH + POPOVER_WIDTH; // 550px total
    
    let mut x = cursor_x;
    let mut y = cursor_y;
    
    // Check right boundary (palette + popover)
    if x + TOTAL_WIDTH > screen_x + screen_width {
        x = screen_x + screen_width - TOTAL_WIDTH;
    }
    
    // Check bottom boundary
    if y + PALETTE_HEIGHT > screen_y + screen_height {
        y = screen_y + screen_height - PALETTE_HEIGHT;
    }
    
    // Check left boundary
    if x < screen_x {
        x = screen_x;
    }
    
    // Check top boundary
    if y < screen_y {
        y = screen_y;
    }
    
    crate::types::WindowPosition { x, y }
}

// Window management commands for blur handling
#[tauri::command]
pub async fn hide_palette_window(app: tauri::AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("palette-window") {
        window.hide().map_err(|e| format!("Failed to hide palette: {}", e))?;
    }
    Ok(())
}

#[tauri::command]
pub async fn focus_palette_window(app: tauri::AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("palette-window") {
        window.set_focus().map_err(|e| format!("Failed to focus palette: {}", e))?;
    }
    Ok(())
}

// Clipboard history commands
#[tauri::command]
pub fn get_clipboard_history(history: tauri::State<ClipboardHistory>) -> Result<Vec<ClipboardItem>, String> {
    Ok(history.get_items())
}

#[tauri::command]
pub async fn paste_clipboard_item(
    app: tauri::AppHandle,
    history: tauri::State<'_, ClipboardHistory>,
    last_active_app: tauri::State<'_, std::sync::Arc<std::sync::Mutex<Option<String>>>>,
    item_id: String,
) -> Result<(), String> {
    // Get the clipboard item
    let item = history
        .get_item_by_id(&item_id)
        .ok_or_else(|| "Clipboard item not found".to_string())?;

    println!("[PasteItem] Pasting item: {}", item.id);

    // Set skip_next_add flag to prevent re-adding this item to history
    history.set_skip_next_add(true);

    // Write item content to clipboard
    app.clipboard()
        .write_text(item.content.clone())
        .map_err(|e| format!("Failed to write to clipboard: {}", e))?;

    // Use the STORED last active app, or fallback to item source, or current active app
    let target_app = {
        let last_app_guard = match last_active_app.lock() {
            Ok(guard) => guard,
            Err(poisoned) => {
                eprintln!("[PasteItem] Mutex poisoned, recovering...");
                poisoned.into_inner()
            }
        };
        if let Some(app_name) = last_app_guard.as_ref() {
            println!("[PasteItem] Using stored last active app: {}", app_name);
            app_name.clone()
        } else if let Some(source) = &item.source_app {
            println!("[PasteItem] Using item source app: {}", source);
            source.clone()
        } else {
            let fallback = automation::get_active_app().unwrap_or_else(|_| "Finder".to_string());
            println!("[PasteItem] Using fallback app: {}", fallback);
            fallback
        }
    };

    println!("[PasteItem] Target app: {}", target_app);

    // Close the window (could be palette or clipboard widget)
    if let Some(window) = app.get_webview_window("palette-window") {
        window.hide().map_err(|e| format!("Failed to hide palette: {}", e))?;
    }
    if let Some(window) = app.get_webview_window("clipboard-window") {
        window.hide().map_err(|e| format!("Failed to hide clipboard: {}", e))?;
    }

    // Execute auto-paste flow in background
    tauri::async_runtime::spawn(async move {
        // Wait a bit for window to close
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        // Restore focus and paste (80-150ms delay as per Electron spec)
        match automation::auto_paste_flow(&target_app, 120) {
            Ok(_) => {
                println!("[PasteItem] ✅ Successfully pasted to application: {}", target_app);
            }
            Err(e) => {
                eprintln!("[PasteItem] ❌ Auto-paste failed: {}", e);
                eprintln!("[PasteItem] Target app: {}", target_app);
                eprintln!("[PasteItem] This may indicate:");
                eprintln!("[PasteItem]   - Application '{}' is not running", target_app);
                eprintln!("[PasteItem]   - Missing accessibility permissions");
                eprintln!("[PasteItem]   - Circuit breaker triggered (too many failures)");
            }
        }
    });

    Ok(())
}


#[tauri::command]
pub fn clear_clipboard_history(history: tauri::State<ClipboardHistory>) -> Result<(), String> {
    history.clear();
    Ok(())
}

#[tauri::command]
pub fn toggle_clipboard_monitor(
    monitor: tauri::State<crate::clipboard::ClipboardMonitor>,
) -> Result<bool, String> {
    let enabled = monitor.toggle();
    Ok(enabled)
}

#[tauri::command]
pub fn get_clipboard_monitor_status(
    monitor: tauri::State<crate::clipboard::ClipboardMonitor>,
) -> Result<bool, String> {
    Ok(monitor.is_enabled())
}

#[tauri::command]
pub fn get_active_app() -> Result<String, String> {
    automation::get_active_app()
}

#[tauri::command]
pub async fn check_accessibility_permissions() -> Result<bool, String> {
    #[cfg(target_os = "macos")]
    {
        use crate::automation::macos::check_accessibility_permissions;
        Ok(check_accessibility_permissions())
    }
    #[cfg(not(target_os = "macos"))]
    Ok(true)
}

// Widget window management
#[tauri::command]
pub async fn show_widget(app: tauri::AppHandle, widget: String) -> Result<(), String> {
    crate::show_widget_window(&app, &widget)
        .map_err(|e| format!("Failed to show widget window: {}", e))
}

#[tauri::command]
pub fn record_command_usage(
    metrics: tauri::State<context::UsageMetrics>,
    command_id: String,
) -> Result<(), String> {
    metrics.record_usage(&command_id);
    Ok(())
}
