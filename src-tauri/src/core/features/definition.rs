//! Definition Lookup feature
//!
//! Provides dictionary lookups with definitions, synonyms, and antonyms.

use crate::shared::types::*;
use super::{FeatureSync, FeatureAsync};
use std::collections::HashMap;
use async_trait::async_trait;
use std::sync::OnceLock;
use reqwest::Client;
use serde::Deserialize;

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
        use crate::shared::types::{DefinitionPayload, DefinitionAction};
        vec![
            CommandItem {
                id: "find_synonyms".to_string(),
                label: "Find Synonyms".to_string(),
                description: Some("Find synonyms for selected word".to_string()),
                action_type: Some(ActionType::DefinitionAction(DefinitionPayload {
                    action: DefinitionAction::FindSynonyms,
                })),
                widget_type: None,
                category: None,
            },
            CommandItem {
                id: "find_antonyms".to_string(),
                label: "Find Antonyms".to_string(),
                description: Some("Find antonyms for selected word".to_string()),
                action_type: Some(ActionType::DefinitionAction(DefinitionPayload {
                    action: DefinitionAction::FindAntonyms,
                })),
                widget_type: None,
                category: None,
            },
            CommandItem {
                id: "brief_definition".to_string(),
                label: "Quick Definition".to_string(),
                description: Some("Get brief definition of selected word".to_string()),
                action_type: Some(ActionType::DefinitionAction(DefinitionPayload {
                    action: DefinitionAction::BriefDefinition,
                })),
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
        
        // Calculate logic response directly
        // We call the command implementation here directly since we are on the backend
        let response = lookup_definition(request).await?;
        
        // Format response based on action type
        use crate::shared::types::DefinitionAction;
        let def_action = match action {
            ActionType::DefinitionAction(payload) => &payload.action,
            _ => return Err(crate::shared::error::AppError::Unknown(crate::shared::errors::ERR_UNSUPPORTED_ACTION.to_string())),
        };
        
        let result = match def_action {
            DefinitionAction::FindSynonyms => {
                if response.synonyms.is_empty() {
                    format!("No synonyms found for '{}'", word)
                } else {
                    format!("Synonyms for '{}': {}", word, response.synonyms.join(", "))
                }
            }
            DefinitionAction::FindAntonyms => {
                if response.antonyms.is_empty() {
                    format!("No antonyms found for '{}'", word)
                } else {
                    format!("Antonyms for '{}': {}", word, response.antonyms.join(", "))
                }
            }
            DefinitionAction::BriefDefinition => {
                if response.definitions.is_empty() {
                    format!("No definition found for '{}'", word)
                } else {
                    let first_def = &response.definitions[0];
                    format!("{} ({}): {}", word, first_def.part_of_speech, first_def.definition)
                }
            }
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

// -- Client Logic Merged Here --

// Lazy static HTTP client to reuse connection pool
static CLIENT: OnceLock<Client> = OnceLock::new();

fn get_client() -> &'static Client {
    CLIENT.get_or_init(|| {
        Client::builder()
            .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
            .build()
            .unwrap_or_else(|_| Client::new())
    })
}

// -- Strict Serde Structs for Free Dictionary API --

#[derive(Debug, Deserialize)]
pub struct FreeDictEntry {
    pub word: String,
    pub phonetic: Option<String>,
    #[serde(default)]
    pub phonetics: Vec<PhoneticEntry>,
    #[serde(default)]
    pub meanings: Vec<Meaning>,
}

#[derive(Debug, Deserialize)]
pub struct PhoneticEntry {
    pub text: Option<String>,
    pub audio: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Meaning {
    #[serde(rename = "partOfSpeech")]
    pub part_of_speech: String,
    #[serde(default)]
    pub definitions: Vec<DefinitionDetail>,
    #[serde(default)]
    pub synonyms: Vec<String>,
    #[serde(default)]
    pub antonyms: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct DefinitionDetail {
    pub definition: String,
    pub example: Option<String>,
    #[serde(default)]
    pub synonyms: Vec<String>,
    #[serde(default)]
    pub antonyms: Vec<String>,
}

async fn lookup_word_client(word: &str) -> crate::shared::error::AppResult<Vec<FreeDictEntry>> {
    let client = get_client();
    let url = format!(
        "https://api.dictionaryapi.dev/api/v2/entries/en/{}",
        urlencoding::encode(word)
    );

    let response = client.get(&url).send().await.map_err(|e| {
        eprintln!("Dictionary API Network Error: {}", e);
        crate::shared::error::AppError::Network(format!("Dictionary API connection failed: {}", e))
    })?;

    if response.status().as_u16() == 404 {
        return Err(crate::shared::error::AppError::Validation(format!("Word '{}' not found", word)));
    }

    if !response.status().is_success() {
        return Err(crate::shared::error::AppError::Network(format!(
            "Dictionary API returned error: {}",
            response.status()
        )));
    }

    let entries = response.json::<Vec<FreeDictEntry>>().await.map_err(|e| {
        eprintln!("Dictionary API Parse Error: {}", e);
        crate::shared::error::AppError::Unknown(format!("Failed to parse definition: {}", e))
    })?;

    Ok(entries)
}

/// Look up word definition using Free Dictionary API (via Client)
#[tauri::command]
pub async fn lookup_definition(request: LookupDefinitionRequest) -> crate::shared::error::AppResult<LookupDefinitionResponse> {
    let entries = lookup_word_client(&request.word).await?;
    
    // Map strict client types to app response types
    if let Some(first_entry) = entries.into_iter().next() {
        let mut all_definitions = Vec::new();
        let mut all_synonyms = Vec::new();
        let mut all_antonyms = Vec::new();
        
        for meaning in first_entry.meanings {
            for def_detail in meaning.definitions.iter().take(3) {
                all_definitions.push(DefinitionEntry {
                    part_of_speech: meaning.part_of_speech.clone(),
                    definition: def_detail.definition.clone(),
                    example: def_detail.example.clone(),
                });
                
                all_synonyms.extend(def_detail.synonyms.clone());
                all_antonyms.extend(def_detail.antonyms.clone());
            }
            
            all_synonyms.extend(meaning.synonyms);
            all_antonyms.extend(meaning.antonyms);
        }
        
        all_synonyms.sort();
        all_synonyms.dedup();
        all_antonyms.sort();
        all_antonyms.dedup();
        
        let phonetic = first_entry.phonetic.or_else(|| {
            first_entry.phonetics.into_iter()
                .find_map(|p| p.text)
        });

        Ok(LookupDefinitionResponse {
            word: first_entry.word,
            phonetic,
            definitions: all_definitions,
            synonyms: all_synonyms,
            antonyms: all_antonyms,
        })
    } else {
         Err(crate::shared::error::AppError::Validation(format!("No definition found for '{}'", request.word)))
    }
}
