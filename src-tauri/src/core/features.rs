//! Feature plugin system
//!
//! Provides a trait-based plugin architecture for features. Each feature
//! (translator, currency, clipboard, etc.) implements the `Feature` trait
//! to provide widgets and actions to the command palette.

use crate::shared::types::{ActionType, CommandItem, ExecuteActionRequest, ExecuteActionResponse};
use crate::core::context::category::{get_action_category, get_widget_category};
use std::collections::HashMap;

pub mod translator;
pub mod currency;
pub mod clipboard;
pub mod unit_converter;
pub mod time_converter;
pub mod definition;
pub mod text_analyser;

/// Feature trait that all features must implement
///
/// This trait defines the interface for features to integrate with the
/// command palette system. Features provide:
/// - Widget commands (e.g., "Open Translator")
/// - Action commands (e.g., "Translate to Spanish")
/// - Action execution logic
pub trait Feature: Send + Sync {
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
    
    /// Execute an action for this feature
    ///
    /// Returns Ok if the action was handled by this feature,
    /// Err if the action is not recognized.
    fn execute_action(
        &self,
        action: &ActionType,
        params: &serde_json::Value,
    ) -> Result<ExecuteActionResponse, String>;
    
    /// Get context boost scores for this feature
    ///
    /// Returns a map of command IDs to boost scores based on the captured text.
    /// Higher scores make commands appear higher in the palette.
    fn get_context_boost(&self, _captured_text: &str) -> HashMap<String, f64> {
        HashMap::new()
    }
}

/// Get all registered features
pub fn get_all_features() -> Vec<Box<dyn Feature>> {
    vec![
        Box::new(translator::TranslatorFeature),
        Box::new(currency::CurrencyFeature),
        Box::new(clipboard::ClipboardFeature),
        Box::new(unit_converter::UnitConverter),
        Box::new(time_converter::TimeConverterFeature),
        Box::new(definition::DefinitionFeature),
        Box::new(text_analyser::TextAnalyserFeature),
    ]
}

/// Get all command items from all features with categories assigned
pub fn get_all_command_items() -> Vec<CommandItem> {
    let mut items = vec![];
    
    for feature in get_all_features() {
        // Get widget commands and assign categories
        let mut widget_cmds = feature.widget_commands();
        for cmd in &mut widget_cmds {
            if let Some(widget_type) = &cmd.widget_type {
                cmd.category = get_widget_category(widget_type);
            }
        }
        items.extend(widget_cmds);
        
        // Get action commands and assign categories
        let mut action_cmds = feature.action_commands();
        for cmd in &mut action_cmds {
            if let Some(action_type) = &cmd.action_type {
                cmd.category = get_action_category(action_type);
            }
        }
        items.extend(action_cmds);
    }
    
    items
}

/// Get context boost from all features
pub fn get_context_boost(captured_text: &str) -> HashMap<String, f64> {
    let mut boost_map = HashMap::new();
    
    for feature in get_all_features() {
        boost_map.extend(feature.get_context_boost(captured_text));
    }
    
    boost_map
}

/// Execute an action across all features
pub fn execute_feature_action(
    request: &ExecuteActionRequest,
) -> Result<ExecuteActionResponse, String> {
    for feature in get_all_features() {
        if let Ok(response) = feature.execute_action(&request.action_type, &request.params) {
            return Ok(response);
        }
    }
    
    Err("Unknown action type".to_string())
}
