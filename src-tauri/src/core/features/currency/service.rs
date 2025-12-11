use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{Arc, Mutex, OnceLock, RwLock},
};

use chrono::{DateTime, Duration, TimeZone, Utc};
use directories::ProjectDirs;
use regex::Regex;
use redb::{Database, ReadableTable, TableDefinition};
use reqwest::Client;
use rust_decimal::Decimal;
use std::str::FromStr;

use crate::shared::error::AppError;

use super::types::{
    CacheSnapshot, ConvertCurrencyRequest, ConvertCurrencyResponse, CurrencyResult, RatesApiResponse,
    StoredRate,
};

const RATES_TABLE: TableDefinition<&str, &str> = TableDefinition::new("currency_rates");
const LAST_UPDATED_KEY: &str = "__last_updated";
const BASE_CURRENCY: &str = "USD";

static SERVICE: OnceLock<Arc<CurrencyService>> = OnceLock::new();

pub struct CurrencyService {
    db: Database,
    http: Client,
    cache: Arc<RwLock<HashMap<String, Decimal>>>,
    last_updated: Arc<Mutex<Option<DateTime<Utc>>>>,
}

impl CurrencyService {
    pub async fn global() -> CurrencyResult<Arc<Self>> {
        if let Some(existing) = SERVICE.get() {
            return Ok(existing.clone());
        }

        let svc = Arc::new(Self::new().await?);
        svc.seed_from_disk()?;
        svc.spawn_refresh_if_stale();
        SERVICE
            .set(svc.clone())
            .map_err(|_| AppError::System("Currency service already initialized".into()))?;
        Ok(svc)
    }

    pub async fn convert(&self, request: ConvertCurrencyRequest) -> CurrencyResult<ConvertCurrencyResponse> {
        // If `from` is not a 3-letter code, attempt fuzzy parsing (e.g., "$10", "1euro")
        let mut amount = request.amount;
        let mut from = request.from.trim().to_ascii_uppercase();
        let to = request.to.trim().to_ascii_uppercase();
        println!("[CurrencyService] convert called: amount={}, from={}, to={}", amount, from, to);

        if !Self::is_valid_code(&from) {
            if let Some((parsed_amount, parsed_code)) = Self::parse_fuzzy_amount(&from) {
                println!("[CurrencyService] Fuzzy parsed 'from' input -> amount={}, from={}", parsed_amount, parsed_code);
                amount = parsed_amount;
                from = parsed_code;
            } else {
                println!("[CurrencyService] Fuzzy parse failed for from='{}'", request.from);
                return Err(AppError::Validation(format!("Currency not supported: {}", request.from)));
            }
        }

        // Ensure cache is populated; network errors only surface when cache is empty.
        if self.cache.read().map_err(|_| AppError::System("cache poisoned".into()))?.is_empty() {
            println!("[CurrencyService] Cache empty; fetching rates");
            self.fetch_and_persist()
                .await
                .map_err(|e| AppError::Network(e.to_string()))?;
        }

        let rates = self
            .cache
            .read()
            .map_err(|_| AppError::System("cache poisoned".into()))?;
        println!("[CurrencyService] Cache size after seed/fetch: {}", rates.len());

        let from_rate = rates
            .get(&from)
            .cloned()
            .ok_or_else(|| AppError::Validation(format!("Currency not supported: {}", from)))?;
        let to_rate = rates
            .get(&to)
            .cloned()
            .ok_or_else(|| AppError::Validation(format!("Currency not supported: {}", request.to)))?;

        // Cross-rate relative to USD: (Amount / Rate_From) * Rate_To
        let cross_rate = Self::triangulate(Decimal::ONE, from_rate, to_rate)?;
        let result = Self::triangulate(amount, from_rate, to_rate)?;

        let last_ts = self
            .last_updated
            .lock()
            .ok()
            .and_then(|g| *g)
            .unwrap_or_else(|| Utc::now());

        println!(
            "[CurrencyService] Conversion complete: {} {} -> {} {} (rate={}, ts={})",
            amount, from, result, to, cross_rate, last_ts
        );

        Ok(ConvertCurrencyResponse {
            result,
            rate: cross_rate,
            timestamp: last_ts.to_rfc3339(),
        })
    }

    async fn new() -> CurrencyResult<Self> {
        let db_path = Self::db_path().await?;
        let db = Database::create(db_path).map_err(|e| AppError::System(e.to_string()))?;
        let http = Client::builder()
            .user_agent("tool-suite-antigravity/currency")
            .build()
            .map_err(|e| AppError::Network(e.to_string()))?;

        Ok(Self {
            db,
            http,
            cache: Arc::new(RwLock::new(HashMap::new())),
            last_updated: Arc::new(Mutex::new(None)),
        })
    }

    fn seed_from_disk(&self) -> CurrencyResult<()> {
        let snapshot = self.read_cache()?;
        {
            let mut cache = self.cache.write().map_err(|_| AppError::System("cache poisoned".into()))?;
            *cache = snapshot.rates;
        }
        println!(
            "[Currency] Seeded cache with {} entries from disk",
            self.cache.read().map(|c| c.len()).unwrap_or(0)
        );
        if let Some(ts) = snapshot.last_updated {
            if let Ok(mut guard) = self.last_updated.lock() {
                *guard = Some(ts);
            }
        }
        Ok(())
    }

    fn spawn_refresh_if_stale(self: &Arc<Self>) {
        let stale = self
            .last_updated
            .lock()
            .ok()
            .and_then(|g| *g)
            .map(|ts| Utc::now() - ts > Duration::hours(24))
            .unwrap_or(true);

        if stale {
            println!("[Currency] Rates older than 24h or missing; scheduling background refresh");
            let svc = Arc::clone(self);
            tauri::async_runtime::spawn(async move {
                println!("[Currency] Background refresh started");
                if let Err(e) = svc.fetch_and_persist().await {
                    eprintln!("[Currency] Background refresh failed: {}", e);
                } else {
                    println!("[Currency] Background rates refreshed");
                }
            });
        }
    }

    async fn fetch_and_persist(&self) -> CurrencyResult<()> {
        let (rates, updated_at) = self.fetch_remote_rates().await?;
        self.write_cache(&rates, updated_at)?;
        self.replace_cache(rates, updated_at);
        Ok(())
    }

    async fn fetch_remote_rates(&self) -> CurrencyResult<(HashMap<String, Decimal>, DateTime<Utc>)> {
        println!("[Currency] Fetching rates from https://open.er-api.com/v6/latest/USD");
        let resp = self
            .http
            .get("https://open.er-api.com/v6/latest/USD")
            .send()
            .await
            .map_err(|e| AppError::Network(e.to_string()))?;

        if !resp.status().is_success() {
            return Err(AppError::Network(format!(
                "Failed to fetch rates: {}",
                resp.status()
            )));
        }

        let json: RatesApiResponse = resp
            .json()
            .await
            .map_err(|e| AppError::Validation(format!("Invalid response: {}", e)))?;

        if json.result.to_lowercase() != "success" {
            return Err(AppError::Network("API reported failure".into()));
        }

        let mut rates = json.rates;
        rates.insert(BASE_CURRENCY.to_string(), Decimal::ONE);

        let ts = json
            .time_last_update_unix
            .and_then(|t| Utc.timestamp_opt(t, 0).single())
            .unwrap_or_else(Utc::now);

        Ok((rates, ts))
    }

    fn write_cache(&self, rates: &HashMap<String, Decimal>, updated_at: DateTime<Utc>) -> CurrencyResult<()> {
        let txn = self.db.begin_write().map_err(|e| AppError::System(e.to_string()))?;
        {
            let mut table = txn
                .open_table(RATES_TABLE)
                .map_err(|e| AppError::System(e.to_string()))?;

            let snapshot_ts = updated_at.timestamp();
            for (code, rate) in rates {
                let payload = StoredRate {
                    rate: *rate,
                    updated_at: snapshot_ts,
                };
                let serialized = serde_json::to_string(&payload)
                    .map_err(|e| AppError::System(e.to_string()))?;
                table
                    .insert(code.as_str(), serialized.as_str())
                    .map_err(|e| AppError::System(e.to_string()))?;
            }

            let ts_string = snapshot_ts.to_string();
            table
                .insert(LAST_UPDATED_KEY, ts_string.as_str())
                .map_err(|e| AppError::System(e.to_string()))?;
        }
        txn.commit()
            .map_err(|e| AppError::System(e.to_string()))
    }

    fn replace_cache(&self, rates: HashMap<String, Decimal>, updated_at: DateTime<Utc>) {
        if let Ok(mut guard) = self.cache.write() {
            *guard = rates;
        }
        if let Ok(mut guard) = self.last_updated.lock() {
            *guard = Some(updated_at);
        }
    }

    fn read_cache(&self) -> CurrencyResult<CacheSnapshot> {
        let mut rates = HashMap::new();
        let mut last_updated: Option<DateTime<Utc>> = None;

        let txn = self.db.begin_read().map_err(|e| AppError::System(e.to_string()))?;
        if let Ok(table) = txn.open_table(RATES_TABLE) {
            for entry in table.iter().map_err(|e| AppError::System(e.to_string()))? {
                let (key, value) = entry.map_err(|e| AppError::System(e.to_string()))?;
                let code = key.value();
                let val = value.value();
                if code == LAST_UPDATED_KEY {
                    if let Ok(parsed) = val.parse::<i64>() {
                        last_updated = Utc.timestamp_opt(parsed, 0).single();
                    }
                    continue;
                }

                if let Ok(stored) = serde_json::from_str::<StoredRate>(val) {
                    rates.insert(code.to_string(), stored.rate);
                    last_updated = last_updated
                        .or_else(|| Utc.timestamp_opt(stored.updated_at, 0).single());
                }
            }
        }

        Ok(CacheSnapshot { rates, last_updated })
    }

    async fn db_path() -> CurrencyResult<PathBuf> {
        let proj_dirs = ProjectDirs::from("com", "Antigravity", "tool-suite-antigravity")
            .ok_or_else(|| AppError::System("Unable to determine data directory".into()))?;
        let mut path = proj_dirs.data_dir().to_path_buf();
        tokio::fs::create_dir_all(&path).await.map_err(|e| AppError::System(e.to_string()))?;
        path.push("currency_rates.redb");
        Ok(path)
    }

    #[inline]
    fn triangulate(amount: Decimal, from_rate: Decimal, to_rate: Decimal) -> CurrencyResult<Decimal> {
        if amount.is_zero() || from_rate == to_rate {
            return Ok(amount);
        }

        if from_rate.is_zero() {
            return Err(AppError::Calculation("Division by zero".into()));
        }

        amount
            .checked_div(from_rate)
            .ok_or_else(|| AppError::Calculation("Division overflow".into()))?
            .checked_mul(to_rate)
            .ok_or_else(|| AppError::Calculation("Multiplication overflow".into()))
    }

    /// Fuzzy parse inputs like "1euro" or "$10" into amount and currency code.
    pub fn parse_natural_input(input: &str) -> Option<(Decimal, String)> {
        static RE: OnceLock<Regex> = OnceLock::new();
        let re = RE.get_or_init(|| Regex::new(r"(?i)^\s*(\D*)(\d+(?:\.\d+)?)(\D*)\s*$").unwrap());

        let caps = re.captures(input)?;
        let prefix = caps.get(1).map(|m| m.as_str()).unwrap_or("").trim();
        let number = caps.get(2)?.as_str();
        let suffix = caps.get(3).map(|m| m.as_str()).unwrap_or("").trim();

        let amount = Decimal::from_str(number).ok()?;

        let detect = |raw: &str| -> Option<String> {
            let token = raw.trim().to_ascii_lowercase();
            if token.is_empty() {
                return None;
            }
            match token.as_str() {
                "$" | "usd" | "dollar" | "dollars" => Some("USD".to_string()),
                "€" | "eur" | "euro" | "euros" => Some("EUR".to_string()),
                "£" | "gbp" | "pound" | "pounds" | "british pound" => Some("GBP".to_string()),
                "¥" | "jpy" | "yen" => Some("JPY".to_string()),
                _ => None,
            }
        };

        let currency = detect(suffix).or_else(|| detect(prefix))?;
        Some((amount, currency))
    }

    /// Fuzzy parse strings with prefix/suffix markers and commas: "$10", "1euro", "€5".
    pub fn parse_fuzzy_amount(input: &str) -> Option<(Decimal, String)> {
        static RE: OnceLock<Regex> = OnceLock::new();
        let re = RE.get_or_init(|| Regex::new(r"(?i)^([^\d\.,]*)([\d\.,]+)([^\d\.,]*)$").unwrap());

        let caps = re.captures(input.trim())?;
        let prefix = caps.get(1).map(|m| m.as_str()).unwrap_or("").trim();
        let number_raw = caps.get(2)?.as_str().replace(',', "");
        let suffix = caps.get(3).map(|m| m.as_str()).unwrap_or("").trim();

        let amount = Decimal::from_str(&number_raw).ok()?;

        let map_token = |raw: &str| -> Option<String> {
            let token = raw.trim().to_ascii_lowercase();
            if token.is_empty() {
                return None;
            }
            match token.as_str() {
                "$" | "usd" | "dollar" | "dollars" => Some("USD".to_string()),
                "€" | "eur" | "euro" | "euros" => Some("EUR".to_string()),
                "£" | "gbp" | "pound" | "pounds" | "british pound" => Some("GBP".to_string()),
                "¥" | "jpy" | "yen" => Some("JPY".to_string()),
                _ => None,
            }
        };

        let currency = map_token(suffix)
            .or_else(|| map_token(prefix))
            .or_else(|| {
                if Self::is_valid_code(prefix) {
                    Some(prefix.to_ascii_uppercase())
                } else if Self::is_valid_code(suffix) {
                    Some(suffix.to_ascii_uppercase())
                } else {
                    // Default to USD when symbol is unknown/empty
                    Some("USD".to_string())
                }
            })?;

        Some((amount, currency))
    }

    #[inline]
    fn is_valid_code(code: &str) -> bool {
        let code = code.trim();
        code.len() == 3 && code.chars().all(|c| c.is_ascii_alphabetic())
    }
}

