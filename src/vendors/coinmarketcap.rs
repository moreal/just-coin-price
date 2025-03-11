use super::ApiVendor;
use reqwest::Client;
use serde_json::Value;

pub struct CoinMarketCapVendor {
    api_key: String,
}

impl CoinMarketCapVendor {
    pub fn new(api_key: String) -> Self {
        Self { api_key }
    }
}

#[async_trait::async_trait]
impl ApiVendor for CoinMarketCapVendor {
    async fn get_price(
        &self,
        ticker: String,
        currency: String,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let client = Client::new();
        let url = format!(
            "https://pro-api.coinmarketcap.com/v2/cryptocurrency/quotes/latest?symbol={}&convert={}",
            ticker,
            currency
        );
        let response = client
            .get(&url)
            .header("X-CMC_PRO_API_KEY", &self.api_key)
            .send()
            .await
            .map_err(|e| format!("Request error: {}", e))?;

        let body = response
            .text()
            .await
            .map_err(|e| format!("Response error: {}", e))?;
        let json: Value =
            serde_json::from_str(&body).map_err(|e| format!("JSON parse error: {}", e))?;

        if json["data"][&ticker].as_array().as_slice().len() > 1 {
            return Err("Maybe duplicated ticker.".into());
        }

        let price = json["data"][&ticker][0]["quote"][&currency]["price"]
            .as_f64()
            .ok_or("Price not found")?;

        Ok(price.to_string())
    }
}
