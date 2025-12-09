use tauri::{AppHandle, Manager};
use crate::shared::types::{ActionType, CommandItem, TextAnalysisRequest, TextAnalysisResponse};
use std::collections::HashMap;
use unicode_segmentation::UnicodeSegmentation;
use async_trait::async_trait;

#[derive(Clone)]
pub struct TextAnalyserFeature;

impl super::FeatureSync for TextAnalyserFeature {
    fn id(&self) -> &str {
        "text_analyser"
    }

    fn widget_commands(&self) -> Vec<CommandItem> {
        vec![
            CommandItem {
                id: "widget_analyser".to_string(),
                label: "Text Analyser".to_string(),
                description: Some("Analyze word count, reading time, etc.".to_string()),
                action_type: None,
                widget_type: Some("text_analyser".to_string()),
                category: Some(crate::core::context::category::ContextCategory::Text),
            }
        ]
    }

    fn action_commands(&self) -> Vec<CommandItem> {
        use crate::shared::types::{TextAnalysisPayload, TextAnalysisAction};
        vec![
            CommandItem {
                id: "count_words".to_string(),
                label: "Count Words".to_string(),
                description: Some("Count words in selected text".to_string()),
                action_type: Some(ActionType::AnalyzeText(TextAnalysisPayload {
                    action: TextAnalysisAction::CountWords,
                })),
                widget_type: None,
                category: Some(crate::core::context::category::ContextCategory::Text),
            },
            CommandItem {
                id: "count_chars".to_string(),
                label: "Count Characters".to_string(),
                description: Some("Count characters (with/without spaces)".to_string()),
                action_type: Some(ActionType::AnalyzeText(TextAnalysisPayload {
                    action: TextAnalysisAction::CountChars,
                })),
                widget_type: None,
                category: Some(crate::core::context::category::ContextCategory::Text),
            },
            CommandItem {
                id: "reading_time".to_string(),
                label: "Reading Time".to_string(),
                description: Some("Estimate reading time".to_string()),
                action_type: Some(ActionType::AnalyzeText(TextAnalysisPayload {
                    action: TextAnalysisAction::ReadingTime,
                })),
                widget_type: None,
                category: Some(crate::core::context::category::ContextCategory::Text),
            },
        ]
    }

    fn get_context_boost(&self, captured_text: &str) -> HashMap<String, f64> {
        let mut boost = HashMap::new();
        // Boost if text is long enough to be worth analyzing
        if captured_text.len() > 50 {
            boost.insert("widget_analyser".to_string(), 40.0);
            boost.insert("reading_time".to_string(), 35.0);
        }
        boost
    }
}

#[async_trait]
impl super::FeatureAsync for TextAnalyserFeature {
    async fn execute_action(&self, action: &ActionType, params: &serde_json::Value) -> crate::shared::error::AppResult<crate::shared::types::ExecuteActionResponse> {
        use crate::shared::types::TextAnalysisAction;
        
        let text = params.get("text")
            .and_then(|t| t.as_str())
            .ok_or_else(|| crate::shared::error::AppError::Validation("Missing 'text' parameter".to_string()))?
            .to_string();

        let analysis = perform_analysis(&text);

        let analysis_action = match action {
            ActionType::AnalyzeText(payload) => &payload.action,
            _ => return Err(crate::shared::error::AppError::Unknown(crate::shared::errors::ERR_UNSUPPORTED_ACTION.to_string())),
        };

        let result_text = match analysis_action {
            TextAnalysisAction::CountWords => format!("{} words", analysis.word_count),
            TextAnalysisAction::CountChars => format!("{} chars ({} without spaces)", analysis.char_count, analysis.char_count_no_spaces),
            TextAnalysisAction::ReadingTime => {
                let mins = (analysis.reading_time_sec / 60.0).floor();
                let secs = (analysis.reading_time_sec % 60.0).round();
                if mins > 0.0 {
                    format!("~{} min {} sec", mins, secs)
                } else {
                    format!("~{} sec", secs)
                }
            },
        };

        Ok(crate::shared::types::ExecuteActionResponse {
            result: result_text,
            metadata: Some(serde_json::to_value(analysis)?),
        })
    }
}

fn perform_analysis(text: &str) -> TextAnalysisResponse {
    let word_count = text.unicode_words().count();
    let char_count = text.chars().count();
    let char_count_no_spaces = text.chars().filter(|c| !c.is_whitespace()).count();
    let grapheme_count = text.graphemes(true).count();
    let line_count = text.lines().count();
    
    // Average reading speed: 200 wpm
    // words / 200 = minutes
    // minutes * 60 = seconds
    let reading_time_sec = if word_count > 0 {
        (word_count as f64 / 200.0) * 60.0
    } else {
        0.0
    };

    TextAnalysisResponse {
        word_count,
        char_count,
        char_count_no_spaces,
        grapheme_count,
        line_count,
        reading_time_sec,
    }
}

#[tauri::command]
pub async fn analyze_text(request: TextAnalysisRequest) -> crate::shared::error::AppResult<TextAnalysisResponse> {
    Ok(perform_analysis(&request.text))
}
