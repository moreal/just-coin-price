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
