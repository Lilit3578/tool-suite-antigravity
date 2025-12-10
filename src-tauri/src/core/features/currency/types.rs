use std::collections::HashMap;

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::Value;

use crate::shared::errors::CommandError;

/// Request payload for currency conversion.
///
/// Amount accepts either a JSON number or string to avoid JS float precision loss;
/// it is parsed into `Decimal` for all calculations.
#[derive(Debug, Clone)]
pub struct ConvertCurrencyRequest {
    pub amount: Decimal,
    pub from: String,
    pub to: String,
}

/// Response payload for currency conversion.
///
/// Decimal fields are serialized as strings to protect precision across the IPC boundary.
#[derive(Debug, Clone, Serialize)]
pub struct ConvertCurrencyResponse {
    #[serde(serialize_with = "serialize_decimal", deserialize_with = "deserialize_decimal")]
    pub result: Decimal,
    #[serde(serialize_with = "serialize_decimal", deserialize_with = "deserialize_decimal")]
    pub rate: Decimal,
    pub timestamp: String,
}

/// Network payload from the open.er-api endpoint.
#[derive(Debug, Deserialize)]
pub struct RatesApiResponse {
    pub result: String,
    pub time_last_update_unix: Option<i64>,
    #[serde(deserialize_with = "deserialize_rates")]
    pub rates: HashMap<String, Decimal>,
}

/// Snapshot used to write/read from the cache database.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredRate {
    #[serde(serialize_with = "serialize_decimal", deserialize_with = "deserialize_decimal")]
    pub rate: Decimal,
    pub updated_at: i64,
}

/// Simple wrapper for cached metadata.
#[derive(Debug, Clone)]
pub struct CacheSnapshot {
    pub rates: HashMap<String, Decimal>,
    pub last_updated: Option<DateTime<Utc>>,
}

pub type CurrencyResult<T> = Result<T, CommandError>;

// ---- Serde helpers ----

impl<'de> Deserialize<'de> for ConvertCurrencyRequest {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct Raw {
            #[serde(deserialize_with = "deserialize_decimal")]
            amount: Decimal,
            from: String,
            to: String,
        }

        let raw = Raw::deserialize(deserializer)?;
        Ok(Self {
            amount: raw.amount,
            from: raw.from.to_uppercase(),
            to: raw.to.to_uppercase(),
        })
    }
}

fn serialize_decimal<S>(value: &Decimal, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&value.to_string())
}

fn deserialize_decimal<'de, D>(deserializer: D) -> Result<Decimal, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum NumOrString {
        Num(f64),
        Str(String),
    }

    match NumOrString::deserialize(deserializer)? {
        NumOrString::Num(n) => Decimal::try_from(n).map_err(serde::de::Error::custom),
        NumOrString::Str(s) => Decimal::from_str_exact(&s).map_err(serde::de::Error::custom),
    }
}

fn deserialize_rates<'de, D>(deserializer: D) -> Result<HashMap<String, Decimal>, D::Error>
where
    D: Deserializer<'de>,
{
    let raw = HashMap::<String, Value>::deserialize(deserializer)?;
    raw.into_iter()
        .map(|(code, value)| {
            let dec = match value {
                Value::Number(num) => num
                    .as_f64()
                    .and_then(|f| Decimal::try_from(f).ok())
                    .ok_or_else(|| serde::de::Error::custom("invalid numeric rate"))?,
                Value::String(s) => Decimal::from_str_exact(&s)
                    .map_err(|e| serde::de::Error::custom(format!("invalid rate string: {}", e)))?,
                _ => return Err(serde::de::Error::custom("unsupported rate type")),
            };
            Ok((code.to_uppercase(), dec))
        })
        .collect()
}

impl<'de> Deserialize<'de> for ConvertCurrencyResponse {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct Raw {
            #[serde(deserialize_with = "deserialize_decimal")]
            result: Decimal,
            #[serde(deserialize_with = "deserialize_decimal")]
            rate: Decimal,
            timestamp: String,
        }

        let raw = Raw::deserialize(deserializer)?;
        Ok(Self {
            result: raw.result,
            rate: raw.rate,
            timestamp: raw.timestamp,
        })
    }
}
