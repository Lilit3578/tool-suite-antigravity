use isolang::Language;
use serde::{Deserialize, Deserializer, Serialize, Serializer};


fn lang_code(lang: &Language) -> String {
    lang.to_639_1()
        .map(|c| c.to_string())
        .unwrap_or_else(|| lang.to_639_3().to_string())
}

#[derive(Debug, Clone)]
pub struct TranslationRequest {
    pub text: String,
    pub source: Option<Language>,
    pub target: Language,
}

#[derive(Debug, Clone)]
pub struct TranslationResponse {
    pub translated: String,
    pub detected: Option<Language>,
    pub cached: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum LanguageStatus {
    Installed = 0,
    NeedsDownload = 1,
    Unsupported = 2,
}

use crate::shared::error::AppError;
pub type TranslatorResult<T> = Result<T, AppError>;

impl Serialize for TranslationRequest {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut s = serializer.serialize_struct("TranslationRequest", 3)?;
        s.serialize_field("text", &self.text)?;
        if let Some(src) = &self.source {
            s.serialize_field("source", &lang_code(src))?;
        }
        s.serialize_field("target", &lang_code(&self.target))?;
        s.end()
    }
}

impl<'de> Deserialize<'de> for TranslationRequest {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct Raw {
            text: String,
            source: Option<String>,
            target: String,
        }
        let raw = Raw::deserialize(deserializer)?;
        let target = Language::from_639_1(raw.target.as_str())
            .or_else(|| Language::from_639_3(raw.target.as_str()))
            .ok_or_else(|| serde::de::Error::custom("invalid target language"))?;
        let source = match raw.source {
            Some(s) if s.to_ascii_lowercase() != "auto" => Some(
                Language::from_639_1(s.as_str())
                    .or_else(|| Language::from_639_3(s.as_str()))
                    .ok_or_else(|| serde::de::Error::custom("invalid source language"))?,
            ),
            _ => None,
        };
        Ok(Self {
            text: raw.text,
            source,
            target,
        })
    }
}

impl Serialize for TranslationResponse {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut s = serializer.serialize_struct("TranslationResponse", 3)?;
        s.serialize_field("translated", &self.translated)?;
        if let Some(det) = &self.detected {
            s.serialize_field("detected", &lang_code(det))?;
        }
        s.serialize_field("cached", &self.cached)?;
        s.end()
    }
}

impl<'de> Deserialize<'de> for TranslationResponse {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct Raw {
            translated: String,
            detected: Option<String>,
            cached: bool,
        }
        let raw = Raw::deserialize(deserializer)?;
        let detected = raw
            .detected
            .and_then(|s| Language::from_639_1(s.as_str()).or_else(|| Language::from_639_3(s.as_str())));
        Ok(Self {
            translated: raw.translated,
            detected,
            cached: raw.cached,
        })
    }
}
