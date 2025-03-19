use poem::Result;
use poem_openapi::param::Query;
use poem_openapi::payload::PlainText;
use poem_openapi::{OpenApi, Tags, param::Path};

use crate::vendors;

#[derive(Tags)]
enum ApiTags {
    #[oai(rename = "Coin Prices")]
    CoinPrices,
}

pub struct Api {
    pub vendor: Box<dyn vendors::ApiVendor>,
    pub allowed_tickers: Vec<String>,
    pub allowed_currencies: Vec<String>,
}

#[OpenApi]
impl Api {
    #[oai(
        path = "/coins/:ticker/price",
        method = "get",
        tag = "ApiTags::CoinPrices"
    )]
    async fn get_coin_price(
        &self,
        ticker: Path<String>,
        currency: Query<Option<String>>,
    ) -> Result<PlainText<String>> {
        let currency = currency.0.unwrap_or("USD".to_string());
        if !self.allowed_tickers.contains(&ticker.0) {
            Err(poem::Error::from_string(
                "Ticker not allowed",
                poem::http::StatusCode::BAD_REQUEST,
            ))
        } else if !self.allowed_currencies.contains(&currency) {
            Err(poem::Error::from_string(
                "Currency not allowed",
                poem::http::StatusCode::BAD_REQUEST,
            ))
        } else {
            Ok(PlainText(
                self.vendor.get_price(ticker.0, currency).await.unwrap(),
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use poem::http::StatusCode;
    use std::collections::HashMap;
    use std::sync::Mutex;

    // Mock ApiVendor for testing
    struct MockVendor {
        // Map of (ticker, currency) -> price
        prices: Mutex<HashMap<(String, String), String>>,
    }

    impl MockVendor {
        fn new() -> Self {
            Self {
                prices: Mutex::new(HashMap::new()),
            }
        }

        fn set_price(&self, ticker: &str, currency: &str, price: &str) {
            self.prices.lock().unwrap().insert(
                (ticker.to_string(), currency.to_string()),
                price.to_string(),
            );
        }
    }

    #[async_trait::async_trait]
    impl vendors::ApiVendor for MockVendor {
        async fn get_price(
            &self,
            ticker: String,
            currency: String,
        ) -> Result<String, Box<dyn std::error::Error>> {
            let prices = self.prices.lock().unwrap();
            if let Some(price) = prices.get(&(ticker.clone(), currency.clone())) {
                Ok(price.clone())
            } else {
                Err(format!("No price found for {}/{}", ticker, currency).into())
            }
        }
    }

    // Helper function to create a test API instance
    fn create_test_api() -> Api {
        let mock_vendor = MockVendor::new();
        mock_vendor.set_price("BTC", "USD", "50000.0");
        mock_vendor.set_price("ETH", "USD", "3000.0");
        mock_vendor.set_price("BTC", "EUR", "45000.0");

        Api {
            vendor: Box::new(mock_vendor),
            allowed_tickers: vec!["BTC".to_string(), "ETH".to_string()],
            allowed_currencies: vec!["USD".to_string(), "EUR".to_string()],
        }
    }

    #[tokio::test]
    async fn test_get_coin_price_success() {
        let api = create_test_api();

        // Test BTC/USD
        let result = api
            .get_coin_price(Path("BTC".to_string()), Query(Some("USD".to_string())))
            .await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().0, "50000.0");

        // Test ETH/USD
        let result = api
            .get_coin_price(Path("ETH".to_string()), Query(Some("USD".to_string())))
            .await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().0, "3000.0");

        // Test BTC/EUR
        let result = api
            .get_coin_price(Path("BTC".to_string()), Query(Some("EUR".to_string())))
            .await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().0, "45000.0");
    }

    #[tokio::test]
    async fn test_default_currency() {
        let api = create_test_api();

        // Test with no currency specified (should default to USD)
        let result = api
            .get_coin_price(Path("BTC".to_string()), Query(None))
            .await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().0, "50000.0");
    }

    #[tokio::test]
    async fn test_invalid_ticker() {
        let api = create_test_api();

        // Test with invalid ticker
        let result = api
            .get_coin_price(Path("WNCG".to_string()), Query(Some("USD".to_string())))
            .await;
        assert!(result.is_err());

        // Extract the error and check its properties
        let err = result.unwrap_err();
        let err_response = err.into_response();
        assert_eq!(err_response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_invalid_currency() {
        let api = create_test_api();

        // Test with invalid currency
        let result = api
            .get_coin_price(Path("BTC".to_string()), Query(Some("KRW".to_string())))
            .await;
        assert!(result.is_err());

        // Extract the error and check its properties
        let err = result.unwrap_err();
        let err_response = err.into_response();
        assert_eq!(err_response.status(), StatusCode::BAD_REQUEST);
    }
}
