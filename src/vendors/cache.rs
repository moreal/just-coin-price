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
            (price.clone(), Utc::now() + chrono::Duration::minutes(45)),
        );

        Ok(price)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};

    struct MockVendor {
        call_count: Arc<AtomicUsize>,
        price: String,
    }

    impl MockVendor {
        fn new(price: &str) -> Self {
            Self {
                call_count: Arc::new(AtomicUsize::new(0)),
                price: price.to_string(),
            }
        }
    }

    #[async_trait::async_trait]
    impl ApiVendor for MockVendor {
        async fn get_price(
            &self,
            _ticker: String,
            _currency: String,
        ) -> Result<String, Box<dyn std::error::Error>> {
            self.call_count.fetch_add(1, Ordering::SeqCst);
            Ok(self.price.clone())
        }
    }

    #[tokio::test]
    async fn test_cache_hit() {
        let mock_vendor = MockVendor::new("100.0");
        let mock_vendor_box = Box::new(mock_vendor);
        let call_count = mock_vendor_box.call_count.clone();

        let cache_vendor = CacheLayerVendor::new(mock_vendor_box);

        let price1 = cache_vendor
            .get_price("BTC".to_string(), "USD".to_string())
            .await
            .unwrap();
        assert_eq!(price1, "100.0");
        assert_eq!(call_count.load(Ordering::SeqCst), 1);

        let price2 = cache_vendor
            .get_price("BTC".to_string(), "USD".to_string())
            .await
            .unwrap();
        assert_eq!(price2, "100.0");
        assert_eq!(call_count.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn test_different_ticker_currency_pairs() {
        let mock_vendor = Box::new(MockVendor::new("100.0"));
        let call_count = mock_vendor.call_count.clone();

        let cache_vendor = CacheLayerVendor::new(mock_vendor);

        cache_vendor
            .get_price("BTC".to_string(), "USD".to_string())
            .await
            .unwrap();
        cache_vendor
            .get_price("ETH".to_string(), "USD".to_string())
            .await
            .unwrap();
        cache_vendor
            .get_price("BTC".to_string(), "EUR".to_string())
            .await
            .unwrap();

        assert_eq!(call_count.load(Ordering::SeqCst), 3);

        cache_vendor
            .get_price("BTC".to_string(), "USD".to_string())
            .await
            .unwrap();
        cache_vendor
            .get_price("ETH".to_string(), "USD".to_string())
            .await
            .unwrap();
        cache_vendor
            .get_price("BTC".to_string(), "EUR".to_string())
            .await
            .unwrap();

        assert_eq!(call_count.load(Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn test_cache_expiration() {
        let mock_vendor = Box::new(MockVendor::new("100.0"));
        let call_count = mock_vendor.call_count.clone();

        let cache_vendor = CacheLayerVendor::new(mock_vendor);

        cache_vendor
            .get_price("BTC".to_string(), "USD".to_string())
            .await
            .unwrap();
        assert_eq!(call_count.load(Ordering::SeqCst), 1);

        {
            let mut cache = cache_vendor.cache.write().await;
            if let Some((_, expires)) = cache.get_mut(&("BTC".to_string(), "USD".to_string())) {
                *expires = Utc::now() - chrono::Duration::minutes(1);
            }
        }

        cache_vendor
            .get_price("BTC".to_string(), "USD".to_string())
            .await
            .unwrap();
        assert_eq!(call_count.load(Ordering::SeqCst), 2);
    }

    #[tokio::test]
    async fn test_vendor_error_propagation() {
        struct ErrorVendor;

        #[async_trait::async_trait]
        impl ApiVendor for ErrorVendor {
            async fn get_price(
                &self,
                _ticker: String,
                _currency: String,
            ) -> Result<String, Box<dyn std::error::Error>> {
                Err("Test error".into())
            }
        }

        let cache_vendor = CacheLayerVendor::new(Box::new(ErrorVendor));

        let result = cache_vendor
            .get_price("BTC".to_string(), "USD".to_string())
            .await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Test error");
    }
}
