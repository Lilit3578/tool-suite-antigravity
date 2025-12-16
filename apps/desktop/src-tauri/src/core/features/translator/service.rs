use std::path::PathBuf;
use keyring::Entry;
use md5::compute;
use redb::{Database, TableDefinition};
use reqwest::Client;
use serde_json;
use urlencoding;


use super::types::{
    TranslationRequest, TranslationResponse, TranslatorResult,
};
use crate::shared::error::AppError;

const CACHE_TABLE: TableDefinition<[u8; 16], &str> = TableDefinition::new("translator_cache");
const KEYRING_SERVICE: &str = "tool-suite-antigravity";
const KEYRING_ACCOUNT: &str = "translator_api_key";

pub struct TranslatorService {
    db: Database,
    http: Client,
}

impl TranslatorService {
    pub fn new() -> TranslatorResult<Self> {
        let cache_path = Self::cache_path();
        let db = Database::create(cache_path).map_err(|e| AppError::System(e.to_string()))?;
        let http = Client::builder()
            .user_agent("tool-suite-antigravity/translator")
            .build()
            .map_err(|e| AppError::Network(e.to_string()))?;

        Ok(Self { db, http })
    }

    fn cache_path() -> PathBuf {
        let mut path = std::env::temp_dir();
        path.push("translator_cache.redb");
        path
    }

    fn hash_request(req: &TranslationRequest) -> [u8; 16] {
        let mut data = Vec::with_capacity(req.text.len() + 16);
        data.extend_from_slice(req.text.as_bytes());
        if let Some(src) = req.source {
            if let Some(code) = src.to_639_1() {
                data.extend_from_slice(code.as_bytes());
            }
        }
        if let Some(code) = req.target.to_639_1() {
            data.extend_from_slice(code.as_bytes());
        }
        compute(data).0
    }

    fn get_api_key(&self) -> TranslatorResult<String> {
        if let Ok(env_key) = std::env::var("DEEPL_API_KEY").or_else(|_| std::env::var("GOOGLE_API_KEY")) {
            if !env_key.trim().is_empty() {
                return Ok(env_key);
            }
        }
        let entry =
            Entry::new(KEYRING_SERVICE, KEYRING_ACCOUNT).map_err(|e| AppError::System(e.to_string()))?;
        match entry.get_password() {
            Ok(p) => Ok(p),
            Err(err) => {
                let msg = err.to_string();
                if msg.to_lowercase().contains("not found") || msg.to_lowercase().contains("no entry") {
                    Err(AppError::Validation("Missing API Key".to_string()))
                } else {
                    Err(AppError::System(msg))
                }
            }
        }
    }

    fn load_from_cache(&self, hash: [u8; 16]) -> TranslatorResult<Option<TranslationResponse>> {
        let read_txn = self.db.begin_read().map_err(|e| AppError::System(e.to_string()))?;
        if let Ok(table) = read_txn.open_table(CACHE_TABLE) {
            if let Some(value) = table.get(hash).map_err(|e| AppError::System(e.to_string()))? {
                let cached = value.value().to_string();
                return Ok(Some(TranslationResponse {
                    translated: cached,
                    detected: None,
                    cached: true,
                }));
            }
        }
        Ok(None)
    }

    fn save_to_cache(&self, hash: [u8; 16], translated: &str) -> TranslatorResult<()> {
        let write_txn = self.db.begin_write().map_err(|e| AppError::System(e.to_string()))?;
        {
            let mut cache = write_txn
                .open_table(CACHE_TABLE)
                .map_err(|e| AppError::System(e.to_string()))?;
            cache
                .insert(hash, translated)
                .map_err(|e| AppError::System(e.to_string()))?;
        }
        write_txn.commit().map_err(|e| AppError::System(e.to_string()))?;
        Ok(())
    }

    async fn fetch_translation(&self, req: &TranslationRequest) -> TranslatorResult<TranslationResponse> {
        let api_key = self.get_api_key()?;
        let target = req
            .target
            .to_639_1()
            .ok_or_else(|| AppError::Validation(format!("Invalid language: {}", req.target)))?
            .to_uppercase();
        let source_code = match req.source {
            Some(lang) => Some(
                lang.to_639_1()
                    .ok_or_else(|| AppError::Validation(format!("Invalid language: {}", lang)))?
                    .to_uppercase(),
            ),
            None => None,
        };

        let mut form: Vec<(&str, String)> = vec![("text", req.text.clone()), ("target_lang", target)];
        if let Some(src) = source_code {
            form.push(("source_lang", src));
        }

        let resp = self
            .http
            .post("https://api-free.deepl.com/v2/translate")
            .header("Authorization", format!("DeepL-Auth-Key {}", api_key))
            .form(&form)
            .send()
            .await
            .map_err(|e| AppError::Network(e.to_string()))?
            .json::<serde_json::Value>()
            .await
            .map_err(|e| AppError::Validation(e.to_string()))?;

        // Safe: Use get() to access nested arrays safely, preventing panic on malformed API responses
        let translated = resp.get("translations")
            .and_then(|v| v.get(0))
            .and_then(|v| v.get("text"))
            .and_then(|v| v.as_str())
            .ok_or_else(|| AppError::Validation("missing translation text".to_string()))?
            .to_string();
        let detected = resp.get("translations")
            .and_then(|v| v.get(0))
            .and_then(|v| v.get("detected_source_language"))
            .and_then(|v| v.as_str())
            .and_then(|code| isolang::Language::from_639_1(code.to_ascii_lowercase().as_str()));

        Ok(TranslationResponse {
            translated,
            detected,
            cached: false,
        })
    }

    pub async fn translate(&self, req: TranslationRequest) -> TranslatorResult<String> {
        // =========================================================
        // PRODUCTION LOGIC (Commented out for Testing)
        // =========================================================
        /*
        // 1. Check Cache
        // ... (Your existing cache logic)
        // 2. Get Secure Key
        let api_key = self.get_api_key().map_err(|e| CommandError::FeatureMissing(e.to_string()))?;
        // 3. Execute Request (DeepL)
        let res = self.client.post("https://api.deepl.com/v2/translate")
            .form(&[
                ("auth_key", api_key),
                ("text", req.text.clone()),
                // ("target_lang", req.target_lang.to_639_1().unwrap_or_default().to_string()),
            ])
            .send()
            .await
            .map_err(|e| CommandError::SystemIO(e.to_string()))?;
        let text = res.text().await.map_err(|e| CommandError::SystemIO(e.to_string()))?;
        return Ok(text);
        */

        // =========================================================
        // BYPASS MODE (Google Free API)
        // =========================================================
        println!("[Translator] Using Google Free API (Bypass Mode)");

        let target_code = req
            .target
            .to_639_1()
            .map(|c| c.to_string())
            .unwrap_or_else(|| req.target.to_639_3().to_string());

        // Unofficial Google Translate Endpoint
        let url = format!(
            "https://translate.googleapis.com/translate_a/single?client=gtx&sl=auto&tl={}&dt=t&q={}",
            target_code,
            urlencoding::encode(&req.text)
        );
        let res = self.http.get(&url)
            .send()
            .await
            .map_err(|e| crate::shared::error::AppError::Network(e.to_string()))?;
        if !res.status().is_success() {
            return Err(crate::shared::error::AppError::Network(format!("Google API Error: {}", res.status())));
        }
        let raw_json: serde_json::Value = res.json().await
            .map_err(|e| crate::shared::error::AppError::Validation(format!("Failed to parse JSON: {}", e)))?;
        // Parse nested array: [[["Translated Text", ...]]]
        let mut result = String::new();
        if let Some(sentences) = raw_json.get(0).and_then(|v| v.as_array()) {
            for sentence in sentences {
                if let Some(segment) = sentence.get(0).and_then(|v| v.as_str()) {
                    result.push_str(segment);
                }
            }
        } else {
            return Err(crate::shared::error::AppError::Validation("Invalid response format from Google".to_string()));
        }
        Ok(result)
    }
}
