//! Currency converter feature
//!
//! Provides currency conversion with 10 major currencies.

use crate::core::context;
use crate::features::currency::{service::CurrencyService, types as currency_types};
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
        
        conversions
            .into_iter()
            .map(|(id, label, action_type)| CommandItem {
                id: id.to_string(),
                label: label.to_string(),
                description: None,
                action_type: Some(action_type),
                widget_type: None,
                category: None, // Will be assigned in get_all_command_items()
            })
            .collect()
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

        // Map known quick actions to explicit targets
        let target_currency = match action {
            ActionType::ConvertUsd => "USD",
            ActionType::ConvertEur => "EUR",
            ActionType::ConvertGbp => "GBP",
            ActionType::ConvertJpy => "JPY",
            ActionType::ConvertAud => "AUD",
            ActionType::ConvertCad => "CAD",
            ActionType::ConvertChf => "CHF",
            ActionType::ConvertCny => "CNY",
            ActionType::ConvertInr => "INR",
            ActionType::ConvertMxn => "MXN",
            _ => {
                println!("[CurrencyFeature] Currency ignoring action: {:?}", action);
                return Err(crate::shared::error::AppError::Unknown(
                    crate::shared::errors::ERR_UNSUPPORTED_ACTION.to_string(),
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
