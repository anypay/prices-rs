use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::env;
use std::collections::HashMap;

#[derive(Deserialize, Debug)]
struct CurrencyData {
    quote: std::collections::HashMap<String, Quote>,
}

#[derive(Deserialize, Debug)]
pub struct ApiResponse {
    pub status: Status,
    pub data: HashMap<String, Vec<CryptoData>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Status {
    pub timestamp: String,
    pub error_code: i32,
    pub error_message: Option<String>,
    pub elapsed: i32,
    pub credit_count: i32,
    pub notice: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CryptoData {
    pub id: i32,
    pub name: String,
    pub symbol: String,
    pub slug: String,
    pub num_market_pairs: i32,
    pub date_added: String,
    pub tags: Vec<Tag>,
    pub max_supply: Option<i64>,
    pub circulating_supply: f64,
    pub total_supply: f64,
    pub is_active: i32,
    pub infinite_supply: bool,
    pub platform: Option<serde_json::Value>,
    pub cmc_rank: i32,
    pub is_fiat: i32,
    pub self_reported_circulating_supply: Option<serde_json::Value>,
    pub self_reported_market_cap: Option<serde_json::Value>,
    pub tvl_ratio: Option<serde_json::Value>,
    pub last_updated: String,
    pub quote: HashMap<String, Quote>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Tag {
    pub slug: String,
    pub name: String,
    pub category: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Quote {
    pub price: f64,
    pub volume_24h: f64,
    pub volume_change_24h: f64,
    pub percent_change_1h: f64,
    pub percent_change_24h: f64,
    pub percent_change_7d: f64,
    pub percent_change_30d: f64,
    pub percent_change_60d: f64,
    pub percent_change_90d: f64,
    pub market_cap: f64,
    pub market_cap_dominance: f64,
    pub fully_diluted_market_cap: f64,
    pub tvl: Option<serde_json::Value>,
    pub last_updated: String,
}

async fn get_price_quote(currency: String) -> Result<HashMap<String, Vec<CryptoData>>, Box<dyn std::error::Error>> {
    if env::var("coinmarketcap_api_key").is_err() {
        println!("coinmarketcap_api_key environment variable must be set");
        return Err("coinmarketcap_api_key not set".into());
    }

    let api_key = env::var("coinmarketcap_api_key")?;
    let client = Client::new();
    let resp = client
        .get("https://pro-api.coinmarketcap.com/v2/cryptocurrency/quotes/latest")
        .query(&[("symbol", &currency)])
        .header("X-CMC_PRO_API_KEY", &api_key)
        .send()
        .await?;

    // Here's where you need to await the future returned by `.send()` before you can call `.json()`
    let api_response: ApiResponse = resp.json().await?;

    Ok(api_response.data)
}

pub async fn get_price(currency: String) -> Result<f64, Box<dyn std::error::Error>> {
    let currency_clone = currency.clone(); // Clone the currency variable

    // Here's where you need to await the future returned by `.send()` before you can call `.json()`
    let api_response: HashMap<String, Vec<CryptoData>> = get_price_quote(currency_clone).await?;

    // Now that you have the ApiResponse, you can access its `data` field
    let currency_data = &api_response.get(&currency).ok_or("Currency not found")?[0];
    let value = currency_data.quote.get("USD").ok_or("USD quote not found")?.price;

    Ok(value)
}
