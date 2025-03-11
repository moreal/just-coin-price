mod coinmarketcap;

pub use coinmarketcap::CoinMarketCapVendor;

#[async_trait::async_trait]
pub trait ApiVendor: Send + Sync {
    async fn get_price(
        &self,
        ticker: String,
        currency: String,
    ) -> Result<String, Box<dyn std::error::Error>>;
}
