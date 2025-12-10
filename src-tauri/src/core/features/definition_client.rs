use std::sync::OnceLock;
use reqwest::Client;
use serde::Deserialize;
use crate::shared::error::AppResult;

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

// -- Public API --

pub async fn lookup_word(word: &str) -> AppResult<Vec<FreeDictEntry>> {
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
