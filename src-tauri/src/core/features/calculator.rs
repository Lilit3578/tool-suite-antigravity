//! Calculator feature with AST-based math evaluation
//!
//! Uses meval crate for safe AST-based expression evaluation.
//! Replaces unsafe string parsing with proper mathematical AST evaluation.

use crate::shared::types::{ActionType, CommandItem, ExecuteActionRequest, ExecuteActionResponse};
use crate::shared::error::AppResult;
use crate::shared::errors::CommandError;
use super::{FeatureSync, FeatureAsync};
use std::collections::HashMap;
use async_trait::async_trait;
use meval::Expr;
use std::str::FromStr;

#[derive(Clone)]
pub struct CalculatorFeature;

impl FeatureSync for CalculatorFeature {
    fn id(&self) -> &str {
        "calculator"
    }
    
    fn widget_commands(&self) -> Vec<CommandItem> {
        vec![CommandItem {
            id: "widget_calculator".to_string(),
            label: "Calculator".to_string(),
            description: Some("Open calculator widget".to_string()),
            action_type: None,
            widget_type: Some("calculator".to_string()),
            category: None,
        }]
    }
    
    fn action_commands(&self) -> Vec<CommandItem> {
        // Calculator doesn't have action commands - it's widget-only
        vec![]
    }
    
    fn get_context_boost(&self, _captured_text: &str) -> HashMap<String, f64> {
        HashMap::new()
    }
}

#[async_trait]
impl FeatureAsync for CalculatorFeature {
    async fn execute_action(
        &self,
        _action: &ActionType,
        _params: &serde_json::Value,
    ) -> AppResult<ExecuteActionResponse> {
        // Calculator doesn't handle actions - it's widget-only
        Err(crate::shared::error::AppError::Feature("Calculator is widget-only".to_string()))
    }
}

/// Evaluate a mathematical expression using AST-based evaluation
/// 
/// Uses meval crate for safe expression parsing and evaluation.
/// Returns Result<f64, AppError> for compatibility with existing code.
pub fn evaluate_expression(expression: &str) -> Result<f64, crate::shared::error::AppError> {
    // Clean the expression (remove whitespace, common math symbols)
    let cleaned = expression.trim()
        .replace("×", "*")
        .replace("÷", "/")
        .replace("−", "-");
    
    // Parse expression into AST
    let expr = Expr::from_str(&cleaned)
        .map_err(|e| crate::shared::error::AppError::Calculation(format!("Failed to parse expression '{}': {}", cleaned, e)))?;
    
    // Evaluate the AST
    let result = expr.eval()
        .map_err(|e| crate::shared::error::AppError::Calculation(format!("Failed to evaluate expression '{}': {}", cleaned, e)))?;
    
    // Check for NaN or Infinity
    if result.is_nan() {
        return Err(crate::shared::error::AppError::Calculation("Result is NaN (Not a Number)".to_string()));
    }
    
    if result.is_infinite() {
        return Err(crate::shared::error::AppError::Calculation("Result is infinite".to_string()));
    }
    
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_simple_addition() {
        assert_eq!(evaluate_expression("2 + 2").unwrap(), 4.0);
    }
    
    #[test]
    fn test_multiplication() {
        assert_eq!(evaluate_expression("3 * 4").unwrap(), 12.0);
    }
    
    #[test]
    fn test_division() {
        assert_eq!(evaluate_expression("10 / 2").unwrap(), 5.0);
    }
    
    #[test]
    fn test_complex_expression() {
        assert_eq!(evaluate_expression("(2 + 3) * 4").unwrap(), 20.0);
    }
    
    #[test]
    fn test_invalid_expression() {
        assert!(evaluate_expression("2 +").is_err());
    }
}
