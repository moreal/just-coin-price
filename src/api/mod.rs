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
    ) -> PlainText<String> {
        PlainText(
            self.vendor
                .get_price(ticker.0, currency.0.unwrap_or("USD".to_string()))
                .await
                .unwrap(),
        )
    }
}
