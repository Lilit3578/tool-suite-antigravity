//! Definition Lookup feature
//!
//! Provides dictionary lookups with definitions, synonyms, and antonyms.

use crate::shared::types::*;
use super::{FeatureSync, FeatureAsync};
use std::collections::HashMap;
use async_trait::async_trait;

#[derive(Clone)]
pub struct DefinitionFeature;

impl FeatureSync for DefinitionFeature {
    fn id(&self) -> &str {
        "definition"
    }
    
    fn widget_commands(&self) -> Vec<CommandItem> {
        vec![CommandItem {
            id: "widget_definition".to_string(),
            label: "Definition Lookup".to_string(),
            description: Some("Look up word definitions".to_string()),
            action_type: None,
            widget_type: Some("definition".to_string()),
            category: None,
        }]
    }
    
    fn action_commands(&self) -> Vec<CommandItem> {
        vec![
            CommandItem {
                id: "find_synonyms".to_string(),
                label: "Find Synonyms".to_string(),
                description: Some("Find synonyms for selected word".to_string()),
                action_type: Some(ActionType::FindSynonyms),
                widget_type: None,
                category: None,
            },
            CommandItem {
                id: "find_antonyms".to_string(),
                label: "Find Antonyms".to_string(),
                description: Some("Find antonyms for selected word".to_string()),
                action_type: Some(ActionType::FindAntonyms),
                widget_type: None,
                category: None,
            },
            CommandItem {
                id: "brief_definition".to_string(),
                label: "Quick Definition".to_string(),
                description: Some("Get brief definition of selected word".to_string()),
                action_type: Some(ActionType::BriefDefinition),
                widget_type: None,
                category: None,
            },
        ]
    }
    
    fn get_context_boost(&self, captured_text: &str) -> HashMap<String, f64> {
        let mut boost_map = HashMap::new();
        
        // Boost if text is a single word (likely a word lookup)
        let words: Vec<&str> = captured_text.split_whitespace().collect();
        if words.len() == 1 && words[0].chars().all(|c| c.is_alphabetic()) {
            boost_map.insert("widget_definition".to_string(), 80.0);
            boost_map.insert("find_synonyms".to_string(), 70.0);
            boost_map.insert("find_antonyms".to_string(), 70.0);
            boost_map.insert("brief_definition".to_string(), 75.0);
        }
        
        boost_map
    }
}

#[async_trait]
impl FeatureAsync for DefinitionFeature {
    async fn execute_action(
        &self,
        action: &ActionType,
        params: &serde_json::Value,
    ) -> crate::shared::error::AppResult<ExecuteActionResponse> {
        let text = params.get("text")
            .and_then(|v| v.as_str())
            .ok_or(crate::shared::error::AppError::Validation("Missing 'text' parameter".to_string()))?;
        
        // Sanitize input: extract first word, remove punctuation
        let word = sanitize_word(text);
        
        if word.is_empty() {
            return Err(crate::shared::error::AppError::Validation("No valid word found in input".to_string()));
        }
        
        let request = LookupDefinitionRequest {
            word: word.clone(),
        };
        
        // Execute lookup synchronously
        let response = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(lookup_definition(request))
        })?;
        
        // Format response based on action type
        let result = match action {
            ActionType::FindSynonyms => {
                if response.synonyms.is_empty() {
                    format!("No synonyms found for '{}'", word)
                } else {
                    format!("Synonyms for '{}': {}", word, response.synonyms.join(", "))
                }
            }
            ActionType::FindAntonyms => {
                if response.antonyms.is_empty() {
                    format!("No antonyms found for '{}'", word)
                } else {
                    format!("Antonyms for '{}': {}", word, response.antonyms.join(", "))
                }
            }
            ActionType::BriefDefinition => {
                if response.definitions.is_empty() {
                    format!("No definition found for '{}'", word)
                } else {
                    let first_def = &response.definitions[0];
                    format!("{} ({}): {}", word, first_def.part_of_speech, first_def.definition)
                }
            }
            _ => return Err(crate::shared::error::AppError::Unknown(crate::shared::errors::ERR_UNSUPPORTED_ACTION.to_string())),
        };
        
        Ok(ExecuteActionResponse {
            result,
            metadata: Some(serde_json::to_value(&response).unwrap_or_default()),
        })
    }
}

/// Sanitize input: extract first word, remove punctuation
fn sanitize_word(text: &str) -> String {
    text.split_whitespace()
        .next()
        .unwrap_or("")
        .chars()
        .filter(|c| c.is_alphabetic())
        .collect::<String>()
        .to_lowercase()
}

/// Look up word definition using Free Dictionary API
#[tauri::command]
pub async fn lookup_definition(request: LookupDefinitionRequest) -> crate::shared::error::AppResult<LookupDefinitionResponse> {
    let client = reqwest::Client::new();
    
    // Free Dictionary API endpoint
    let url = format!(
        "https://api.dictionaryapi.dev/api/v2/entries/en/{}",
        urlencoding::encode(&request.word)
    );
    
    match client.get(&url)
        .header("User-Agent", "Mozilla/5.0")
        .send()
        .await
    {
        Ok(response) => {
            if response.status().is_success() {
                match response.json::<serde_json::Value>().await {
                    Ok(json) => {
                        // Parse the API response
                        if let Some(entries) = json.as_array() {
                            if let Some(first_entry) = entries.first() {
                                let word = first_entry.get("word")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or(&request.word)
                                    .to_string();
                                
                                let phonetic = first_entry.get("phonetic")
                                    .and_then(|v| v.as_str())
                                    .map(|s| s.to_string());
                                
                                let mut definitions = Vec::new();
                                let mut all_synonyms = Vec::new();
                                let mut all_antonyms = Vec::new();
                                
                                if let Some(meanings) = first_entry.get("meanings").and_then(|v| v.as_array()) {
                                    for meaning in meanings {
                                        let part_of_speech = meaning.get("partOfSpeech")
                                            .and_then(|v| v.as_str())
                                            .unwrap_or("unknown")
                                            .to_string();
                                        
                                        // Extract definitions
                                        if let Some(defs) = meaning.get("definitions").and_then(|v| v.as_array()) {
                                            for def in defs.iter().take(3) { // Limit to 3 definitions per part of speech
                                                let definition = def.get("definition")
                                                    .and_then(|v| v.as_str())
                                                    .unwrap_or("")
                                                    .to_string();
                                                
                                                let example = def.get("example")
                                                    .and_then(|v| v.as_str())
                                                    .map(|s| s.to_string());
                                                
                                                definitions.push(DefinitionEntry {
                                                    part_of_speech: part_of_speech.clone(),
                                                    definition,
                                                    example,
                                                });
                                                
                                                // Extract nested synonyms
                                                if let Some(syns) = def.get("synonyms").and_then(|v| v.as_array()) {
                                                    for syn in syns {
                                                        if let Some(s) = syn.as_str() {
                                                            all_synonyms.push(s.to_string());
                                                        }
                                                    }
                                                }
                                                
                                                // Extract nested antonyms
                                                if let Some(ants) = def.get("antonyms").and_then(|v| v.as_array()) {
                                                    for ant in ants {
                                                        if let Some(a) = ant.as_str() {
                                                            all_antonyms.push(a.to_string());
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                        
                                        // Extract synonyms
                                        if let Some(synonyms) = meaning.get("synonyms").and_then(|v| v.as_array()) {
                                            for syn in synonyms {
                                                if let Some(s) = syn.as_str() {
                                                    all_synonyms.push(s.to_string());
                                                }
                                            }
                                        }
                                        
                                        // Extract antonyms
                                        if let Some(antonyms) = meaning.get("antonyms").and_then(|v| v.as_array()) {
                                            for ant in antonyms {
                                                if let Some(a) = ant.as_str() {
                                                    all_antonyms.push(a.to_string());
                                                }
                                            }
                                        }
                                    }
                                }
                                
                                // Deduplicate synonyms and antonyms
                                all_synonyms.sort();
                                all_synonyms.dedup();
                                all_antonyms.sort();
                                all_antonyms.dedup();
                                
                                return Ok(LookupDefinitionResponse {
                                    word,
                                    phonetic,
                                    definitions,
                                    synonyms: all_synonyms,
                                    antonyms: all_antonyms,
                                });
                            }
                        }
                        
                        Err(crate::shared::error::AppError::Validation(format!("No definition found for '{}'", request.word)))
                    }
                    Err(e) => {
                        eprintln!("Failed to parse definition response: {}", e);
                        Err(crate::shared::error::AppError::Unknown(format!("Failed to parse definition for '{}': {}", request.word, e)))
                    }
                }
            } else if response.status().as_u16() == 404 {
                Err(crate::shared::error::AppError::Validation(format!("Word '{}' not found in dictionary", request.word)))
            } else {
                eprintln!("Dictionary API returned error: {}", response.status());
                Err(crate::shared::error::AppError::Network(format!("Dictionary API error for '{}'", request.word)))
            }
        }
        Err(e) => {
            eprintln!("Dictionary API request failed: {}", e);
            Err(crate::shared::error::AppError::Network("Failed to connect to dictionary API".to_string()))
        }
    }
}
