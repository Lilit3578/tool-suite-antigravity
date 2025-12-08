pub mod service;
pub mod types;

use crate::shared::errors::CommandError;

use self::{service::CurrencyService, types::{ConvertCurrencyRequest, ConvertCurrencyResponse}};

#[tauri::command]
pub async fn convert_currency(request: ConvertCurrencyRequest) -> Result<ConvertCurrencyResponse, CommandError> {
    let service = CurrencyService::global()?;
    service.convert(request).await
}
