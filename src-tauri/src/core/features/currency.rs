//! Currency converter feature
//!
//! Provides currency conversion with 10 major currencies.

use crate::core::context;
pub mod service;
pub mod types;
use self::service::CurrencyService;
use self::types as currency_types;
use crate::shared::error::AppError;
use crate::shared::types::*;
use super::{FeatureAsync, FeatureSync};
use async_trait::async_trait;
use rust_decimal::Decimal;
use std::collections::HashMap;
use std::str::FromStr;

#[derive(Clone)]
pub struct CurrencyFeature;

impl FeatureSync for CurrencyFeature {
    fn id(&self) -> &str {
        "currency"
    }
    
    fn widget_commands(&self) -> Vec<CommandItem> {
        vec![CommandItem {
            id: "widget_currency".to_string(),
            label: "Currency Converter".to_string(),
            description: Some("Open currency converter widget".to_string()),
            action_type: None,
            widget_type: Some("currency".to_string()),
            category: None, // Will be assigned in get_all_command_items()
        }]
    }
    
    fn action_commands(&self) -> Vec<CommandItem> {
        // Phase 2: Generate commands dynamically from currency list
        // This replaces 10 hardcoded CommandItem definitions
        const CURRENCIES: &[(&str, &str)] = &[
            ("USD", "US Dollar"),
            ("EUR", "Euro"),
            ("GBP", "British Pound"),
            ("JPY", "Japanese Yen"),
            ("AUD", "Australian Dollar"),
            ("CAD", "Canadian Dollar"),
            ("CHF", "Swiss Franc"),
            ("CNY", "Chinese Yuan"),
            ("INR", "Indian Rupee"),
            ("MXN", "Mexican Peso"),
        ];
        
        CURRENCIES.iter().map(|(code, name)| CommandItem {
            id: format!("convert_{}", code.to_lowercase()),
            label: format!("Convert to {}", name),
            description: None,
            action_type: Some(ActionType::ConvertCurrency(CurrencyPayload {
                target_currency: code.to_string(),
            })),
            widget_type: None,
            category: None, // Will be assigned by get_action_category
        }).collect()
    }
    
    fn get_context_boost(&self, captured_text: &str) -> HashMap<String, f64> {
        let mut boost_map = HashMap::new();
        
        // Detect currency in text
        if let Some(currency_info) = context::detect_currency(captured_text) {
            println!("[Context] Detected currency: {} {}", currency_info.amount, currency_info.currency_code);
            
            // Boost currency widget
            boost_map.insert("widget_currency".to_string(), 100.0);
            
            // Boost currency conversion actions
            for id in &[
                "convert_usd", "convert_eur", "convert_gbp", "convert_jpy",
                "convert_aud", "convert_cad", "convert_chf", "convert_cny",
                "convert_inr", "convert_mxn",
            ] {
                boost_map.insert(id.to_string(), 80.0);
            }
        }
        
        boost_map
    }
}

#[async_trait]
impl FeatureAsync for CurrencyFeature {
    async fn execute_action(
        &self,
        action: &ActionType,
        params: &serde_json::Value,
    ) -> crate::shared::error::AppResult<ExecuteActionResponse> {
        println!("!! CURRENCY EXECUTE CALLED with {:?}", action);
        println!("!! CURRENCY PARAMS: {}", params);

        // Phase 4: Only handle new ConvertCurrency variant
        let target_currency = match action {
            ActionType::ConvertCurrency(payload) => payload.target_currency.as_str(),
            _ => {
                println!("[CurrencyFeature] Currency ignoring action: {:?}", action);
                return Err(crate::shared::error::AppError::Unknown(
                    "Unsupported action".to_string(),
                ));
            }
        };

        println!("[CurrencyFeature] DEBUG: Target currency resolved to {}", target_currency);
        
        let text = params.get("text")
            .and_then(|v| v.as_str())
            .unwrap_or("100");
        println!("[CurrencyFeature] DEBUG: Input text = '{}'", text);
        
        // Attempt fuzzy parse like "1euro" or "$10"
        let (amount, from) = if let Some((amt, code)) = CurrencyService::parse_natural_input(text) {
            println!("[CurrencyFeature] DEBUG: parse_natural_input succeeded -> amount={}, from={}", amt, code);
            (amt, code)
        } else {
            let amount_str = text
                .chars()
                .filter(|c| c.is_numeric() || *c == '.')
                .collect::<String>();
            println!("[CurrencyFeature] DEBUG: Fallback numeric parse string='{}'", amount_str);
            let amt = Decimal::from_str(&amount_str)
                .or_else(|_| Decimal::from_str("100"))
                .unwrap_or_else(|_| Decimal::from(100u32));
            let from = params
                .get("from")
                .and_then(|v| v.as_str())
                .unwrap_or("USD")
                .to_string();
            println!("[CurrencyFeature] DEBUG: Parsed fallback amount={}, from={}", amt, from);
            (amt, from)
        };

        let convert_request = currency_types::ConvertCurrencyRequest {
            amount,
            from,
            to: target_currency.to_string(),
        };
        println!("[CurrencyFeature] DEBUG: convert_request = amount={}, from={}, to={}", convert_request.amount, convert_request.from, convert_request.to);
        
        // Execute conversion asynchronously
        let service = CurrencyService::global().await
            .map_err(|e| AppError::Unknown(e.to_string()))?;
        println!("[CurrencyFeature] DEBUG: CurrencyService acquired");

        let response = service
            .convert(convert_request)
            .await
            .map_err(|e| {
                println!("[CurrencyFeature] ERROR: convert failed: {}", e);
                AppError::from(e)
            })?;
        println!("[CurrencyFeature] DEBUG: convert response: result={}, rate={}, ts={}", response.result, response.rate, response.timestamp);
        
        Ok(ExecuteActionResponse {
            result: format!("{} {}", response.result, target_currency),
            metadata: Some(serde_json::json!({
                "rate": response.rate,
                "timestamp": response.timestamp,
            })),
        })
    }
}

// Legacy function retained for backward compatibility; delegates to the new service.
#[tauri::command]
pub async fn convert_currency(
    request: ConvertCurrencyRequest,
) -> crate::shared::error::AppResult<currency_types::ConvertCurrencyResponse> {
    let service = CurrencyService::global().await
        .map_err(|e| AppError::Unknown(e.to_string()))?;

    // Try strict parse first; fallback to fuzzy parsing on failure
    let mut amount = Decimal::from_str(&request.amount)
        .or_else(|_| Decimal::from_str("0"))
        .unwrap_or_else(|_| Decimal::ZERO);
    let mut from = request.from.clone();

    if amount.is_zero() {
        if let Some((amt, code)) = CurrencyService::parse_fuzzy_amount(&request.amount) {
            amount = amt;
            from = code;
        }
    }

    let response = service
        .convert(currency_types::ConvertCurrencyRequest {
            amount,
            from,
            to: request.to.clone(),
        })
        .await
        .map_err(AppError::from)?;

    Ok(response)
}
