//! Action validation module
//!
//! Enforces strict business rules: prevents invalid conversions
//! (e.g., "5km" cannot be converted to USD).

use crate::shared::types::ActionType;
use crate::core::context::category::{ContextCategory, get_action_category};

/// Validate that an action is permitted for the given text category
/// 
/// Rules:
/// - Category::Length can only execute Length actions
/// - Category::Currency can only execute Currency actions
/// - Category::Volume can execute Volume OR CrossCategory (Volume <-> Mass) actions
/// - Category::Mass can execute Mass OR CrossCategory (Mass <-> Volume) actions
/// - Category::Text can only execute Text actions
/// - Category::Temperature can only execute Temperature actions
/// - Category::Speed can only execute Speed actions
pub fn validate_action(
    text_category: &ContextCategory,
    action: &ActionType,
) -> Result<(), String> {
    let action_category = get_action_category(action)
        .ok_or_else(|| "Unknown action type".to_string())?;
    
    // Check if action is permitted for this text category
    let is_permitted = match text_category {
        ContextCategory::Length => {
            action_category == ContextCategory::Length
        }
        ContextCategory::Currency => {
            action_category == ContextCategory::Currency
        }
        ContextCategory::Volume => {
            // Volume can execute Volume actions OR Mass actions (for cross-category conversions)
            // Generic actions like ConvertToG, ConvertToKG are now smart enough to handle Volume→Mass
            // Strictly allow: Volume ↔ ActionType::ConvertTo[MassUnit]
            action_category == ContextCategory::Volume || 
            action_category == ContextCategory::Mass
        }
        ContextCategory::Mass => {
            // Mass can execute Mass actions OR Volume actions (for cross-category conversions)
            // Generic actions like ConvertToML, ConvertToL are now smart enough to handle Mass→Volume
            // Strictly allow: Mass ↔ ActionType::ConvertTo[VolumeUnit]
            action_category == ContextCategory::Mass ||
            action_category == ContextCategory::Volume
        }
        ContextCategory::Text => {
            // Text context allows ALL actions (translate, convert currency if numeric, convert units, etc.)
            // This provides maximum flexibility - users can do anything with text
            // If text contains a number, they can convert it; if it's words, they can translate it
            true
        }
        ContextCategory::Temperature => {
            action_category == ContextCategory::Temperature
        }
        ContextCategory::Speed => {
            action_category == ContextCategory::Speed
        }
        ContextCategory::Time => {
            action_category == ContextCategory::Time
        }
        ContextCategory::General => {
            // General category allows all actions (fallback)
            true
        }
    };
    
    if !is_permitted {
        // Generate list of valid actions for this category
        let valid_actions = get_valid_actions_for_category(text_category);
        return Err(format!(
            "This conversion is not permitted. Try {}.",
            valid_actions
        ));
    }
    
    Ok(())
}

/// Get a formatted list of valid actions for a category
fn get_valid_actions_for_category(category: &ContextCategory) -> String {
    let actions = match category {
        ContextCategory::Length => vec![
            "Convert to Millimeters",
            "Convert to Centimeters",
            "Convert to Meters",
            "Convert to Kilometers",
            "Convert to Inches",
            "Convert to Feet",
            "Convert to Yards",
            "Convert to Miles",
        ],
        ContextCategory::Mass => vec![
            "Convert to Milligrams",
            "Convert to Grams",
            "Convert to Kilograms",
            "Convert to Ounces",
            "Convert to Pounds",
            "Convert Mass to Milliliters",
            "Convert Mass to Liters",
            "Convert Mass to Fluid Ounces",
            "Convert Mass to Cups",
            "Convert Mass to Gallons",
        ],
        ContextCategory::Volume => vec![
            "Convert to Milliliters",
            "Convert to Liters",
            "Convert to Fluid Ounces",
            "Convert to Cups",
            "Convert to Gallons",
            "Convert Volume to Grams",
            "Convert Volume to Kilograms",
            "Convert Volume to Ounces",
            "Convert Volume to Pounds",
        ],
        ContextCategory::Currency => vec![
            "Convert to US Dollar (USD)",
            "Convert to Euro (EUR)",
            "Convert to British Pound (GBP)",
            "Convert to Japanese Yen (JPY)",
            "Convert to Australian Dollar (AUD)",
            "Convert to Canadian Dollar (CAD)",
            "Convert to Swiss Franc (CHF)",
            "Convert to Chinese Yuan (CNY)",
            "Convert to Indian Rupee (INR)",
            "Convert to Mexican Peso (MXN)",
        ],
        ContextCategory::Text => vec![
            "Translate to English",
            "Translate to Italian",
            "Translate to Spanish",
            "Translate to French",
            "Translate to German",
            "Translate to Chinese",
            "Translate to Japanese",
            // ... (all translation actions)
        ],
        ContextCategory::Temperature => vec![
            "Convert to Celsius",
            "Convert to Fahrenheit",
        ],
        ContextCategory::Speed => vec![
            "Convert to Kilometers/Hour",
            "Convert to Miles/Hour",
        ],
        ContextCategory::Time => vec![
            "Convert to Hours",
            "Convert to Minutes",
            "Convert to Seconds",
            "Convert to Milliseconds",
        ],
        ContextCategory::General => vec!["Any conversion"],
    };
    
    if actions.len() <= 3 {
        format!("[{}]", actions.join(", "))
    } else {
        format!("[{}]", actions.iter().take(3).cloned().collect::<Vec<_>>().join(", ") + ", ...")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shared::types::{CurrencyPayload};

    #[test]
    fn test_validate_length_to_length() {
        assert!(validate_action(
            &ContextCategory::Length,
            &ActionType::ConvertUnit { target: "km".to_string() }
        ).is_ok());
    }

    #[test]
    fn test_validate_length_to_currency() {
        assert!(validate_action(
            &ContextCategory::Length,
            &ActionType::ConvertCurrency(CurrencyPayload { target_currency: "USD".to_string() })
        ).is_err());
    }

    #[test]
    fn test_validate_volume_to_mass() {
        // Validation logic allows Volume -> Mass cross-category
        assert!(validate_action(
            &ContextCategory::Volume,
            &ActionType::ConvertUnit { target: "g".to_string() }
        ).is_ok());
    }

    #[test]
    fn test_validate_mass_to_volume() {
        // Validation logic allows Mass -> Volume cross-category
        assert!(validate_action(
            &ContextCategory::Mass,
            &ActionType::ConvertUnit { target: "l".to_string() }
        ).is_ok());
    }

    #[test]
    fn test_validate_currency_to_currency() {
        assert!(validate_action(
            &ContextCategory::Currency,
            &ActionType::ConvertCurrency(CurrencyPayload { target_currency: "USD".to_string() })
        ).is_ok());
    }
}

