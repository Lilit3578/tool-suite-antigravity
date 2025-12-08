//! Feature plugin system with enum dispatch
//!
//! Uses enum_dispatch for zero-cost abstraction and static dispatch.
//! Replaces trait objects (Box<dyn Feature>) with enum variants for better performance.

use crate::shared::types::{ActionType, CommandItem, ExecuteActionRequest, ExecuteActionResponse};
use crate::core::context::category::{get_action_category, get_widget_category};
use std::collections::HashMap;
use enum_dispatch::enum_dispatch;

pub mod translator;
pub mod currency;
pub mod clipboard;
pub mod unit_converter;
pub mod time_converter;
pub mod definition;
pub mod text_analyser;
pub mod calculator;

use async_trait::async_trait;

/// Sync methods trait for enum_dispatch
/// 
/// enum_dispatch works with sync methods only.
/// Async methods are handled separately via async_trait.
#[enum_dispatch]
pub trait FeatureSync: Send + Sync {
    /// Unique identifier for this feature
    fn id(&self) -> &str;
    
    /// Get widget commands for this feature
    ///
    /// Widget commands open the feature's UI window.
    /// Example: "Translator" widget command opens the translator window.
    fn widget_commands(&self) -> Vec<CommandItem>;
    
    /// Get action commands for this feature
    ///
    /// Action commands perform immediate actions without opening a window.
    /// Example: "Translate to Spanish" translates selected text.
    fn action_commands(&self) -> Vec<CommandItem>;
    
    /// Get context boost scores for this feature
    ///
    /// Returns a map of command IDs to boost scores based on the captured text.
    /// Higher scores make commands appear higher in the palette.
    fn get_context_boost(&self, captured_text: &str) -> HashMap<String, f64>;
}

/// Async methods trait (separate from enum_dispatch)
#[async_trait]
pub trait FeatureAsync: Send + Sync {
    /// Execute an action for this feature
    ///
    /// Returns Ok if the action was handled by this feature,
    /// Err if the action is not recognized or execution failed.
    async fn execute_action(
        &self,
        action: &ActionType,
        params: &serde_json::Value,
    ) -> crate::shared::error::AppResult<ExecuteActionResponse>;
}

/// Combined Feature trait (for convenience in implementations)
#[async_trait]
pub trait Feature: FeatureSync + FeatureAsync {}

/// AppFeature enum with enum_dispatch for static dispatch
///
/// This replaces Vec<Box<dyn Feature>> with Vec<AppFeature> for zero-cost abstraction.
/// All feature types are known at compile time, enabling better optimization.
/// 
/// enum_dispatch handles sync methods (FeatureSync), async methods handled separately.
#[enum_dispatch(FeatureSync)]
pub enum AppFeature {
    Translator(translator::TranslatorFeature),
    Currency(currency::CurrencyFeature),
    Clipboard(clipboard::ClipboardFeature),
    UnitConverter(unit_converter::UnitConverterFeature),
    TimeConverter(time_converter::TimeConverterFeature),
    Definition(definition::DefinitionFeature),
    TextAnalyser(text_analyser::TextAnalyserFeature),
    Calculator(calculator::CalculatorFeature),
}

/// Get all registered features (static dispatch, no trait objects)
pub fn get_all_features() -> Vec<AppFeature> {
    vec![
        AppFeature::Translator(translator::TranslatorFeature),
        AppFeature::Currency(currency::CurrencyFeature),
        AppFeature::Clipboard(clipboard::ClipboardFeature),
        AppFeature::UnitConverter(unit_converter::UnitConverterFeature),
        AppFeature::TimeConverter(time_converter::TimeConverterFeature),
        AppFeature::Definition(definition::DefinitionFeature),
        AppFeature::TextAnalyser(text_analyser::TextAnalyserFeature),
        AppFeature::Calculator(calculator::CalculatorFeature),
    ]
}

/// Get all command items from all features with categories assigned
pub fn get_all_command_items() -> Vec<CommandItem> {
    let mut items = vec![];
    
    println!("[get_all_command_items] Getting commands from {} features", get_all_features().len());
    
    for feature in get_all_features() {
        // Use enum_dispatch generated methods directly
        let mut widget_cmds = feature.widget_commands();
        println!("[get_all_command_items] Feature {}: {} widget commands", feature.id(), widget_cmds.len());
        for cmd in &mut widget_cmds {
            if let Some(widget_type) = &cmd.widget_type {
                cmd.category = get_widget_category(widget_type);
            }
        }
        items.extend(widget_cmds);
        
        // Use enum_dispatch generated methods directly
        let mut action_cmds = feature.action_commands();
        println!("[get_all_command_items] Feature {}: {} action commands", feature.id(), action_cmds.len());
        for cmd in &mut action_cmds {
            if let Some(action_type) = &cmd.action_type {
                cmd.category = get_action_category(action_type);
            }
        }
        items.extend(action_cmds);
    }
    
    println!("[get_all_command_items] Total commands: {}", items.len());
    items
}

/// Get context boost from all features
pub fn get_context_boost(captured_text: &str) -> HashMap<String, f64> {
    let mut boost_map = HashMap::new();
    
    for feature in get_all_features() {
        // Use enum_dispatch generated method directly
        boost_map.extend(feature.get_context_boost(captured_text));
    }
    
    boost_map
}

/// Execute an action across all features
pub async fn execute_feature_action(
    request: &ExecuteActionRequest,
) -> crate::shared::error::AppResult<ExecuteActionResponse> {
    for feature in get_all_features() {
        // Use manual dispatch for async methods (enum_dispatch doesn't support async)
        let result = match &feature {
            AppFeature::Translator(f) => f.execute_action(&request.action_type, &request.params).await,
            AppFeature::Currency(f) => f.execute_action(&request.action_type, &request.params).await,
            AppFeature::Clipboard(f) => f.execute_action(&request.action_type, &request.params).await,
            AppFeature::UnitConverter(f) => f.execute_action(&request.action_type, &request.params).await,
            AppFeature::TimeConverter(f) => f.execute_action(&request.action_type, &request.params).await,
            AppFeature::Definition(f) => f.execute_action(&request.action_type, &request.params).await,
            AppFeature::TextAnalyser(f) => f.execute_action(&request.action_type, &request.params).await,
            AppFeature::Calculator(f) => f.execute_action(&request.action_type, &request.params).await,
        };
        match result {
            Ok(response) => return Ok(response),
             // We'll assume implementations return AppError::Unknown("Unsupported action type") or similar 
             // for now, OR we just check if it returns ANY error?
             // No, if a feature TRIES to handle it and FAILS (e.g. network error), we should return that error.
             // But if it just doesn't know the action, we continue.
             
             // To simplify: we'll check message for now, but ideally we add `AppError::UnsupportedAction` variant later.
             // Or we rely on `utils.ts` equivalent in Rust? 
             
             // Actually, `core::shared::errors::ERR_UNSUPPORTED_ACTION` exists.
            Err(e) => {
                 let err_str = e.to_string();
                 if err_str == crate::shared::errors::ERR_UNSUPPORTED_ACTION {
                    continue;
                 }
                 // If it's a real error (not just unsupported), we could return it?
                 // But multiple features might share action types? No, ActionType variants are unique usually.
                 // So if a feature claims to handle it (by not returning Unsupported), but fails, we stop.
                 return Err(e);
            }
        }
    }
    
    // If we get here, no feature handled it
    Err(crate::shared::error::AppError::Feature("Unknown action type".to_string()))
}
