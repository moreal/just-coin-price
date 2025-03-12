use super::ApiVendor;
use chrono::Utc;
use tokio::sync::RwLock;

type Cache = RwLock<std::collections::HashMap<(String, String), (String, chrono::DateTime<Utc>)>>;

pub struct CacheLayerVendor {
    vendor: Box<dyn ApiVendor>,
    cache: Cache,
}

impl CacheLayerVendor {
    pub fn new(vendor: Box<dyn ApiVendor>) -> Self {
        Self {
            vendor,
            cache: Default::default(),
        }
    }
}

#[async_trait::async_trait]
impl ApiVendor for CacheLayerVendor {
    async fn get_price(
        &self,
        ticker: String,
        currency: String,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let cache_read = self.cache.read().await;
        if let Some((price, expires)) = cache_read.get(&(ticker.clone(), currency.clone())) {
            if expires > &Utc::now() {
                return Ok(price.clone());
            }
        }
        drop(cache_read);

        let price = self
            .vendor
            .get_price(ticker.clone(), currency.clone())
            .await?;
        self.cache.write().await.insert(
            (ticker.clone(), currency.clone()),
            (price.clone(), Utc::now() + chrono::Duration::minutes(10)),
        );

        Ok(price)
    }
}
