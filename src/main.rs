use poem::EndpointExt;
use poem::middleware::Cors;
use poem::{Route, Server, listener::TcpListener};
use poem_openapi::param::Query;
use poem_openapi::payload::PlainText;
use poem_openapi::{OpenApi, OpenApiService, Tags, param::Path};

mod vendors;

#[derive(Tags)]
enum ApiTags {
    #[oai(rename = "Coin Prices")]
    CoinPrices,
}

struct Api {
    vendor: Box<dyn vendors::ApiVendor>,
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

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    if std::env::var_os("RUST_LOG").is_none() {
        unsafe {
            std::env::set_var("RUST_LOG", "poem=debug");
        }
    }

    let cmc_api_key = std::env::var("CMC_API_KEY")
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::NotFound, e))?;

    tracing_subscriber::fmt::init();

    let api_service = OpenApiService::new(
        Api {
            vendor: Box::new(vendors::CacheLayerVendor::new(Box::new(
                vendors::CoinMarketCapVendor::new(cmc_api_key),
            ))),
        },
        "Just Coin Price",
        "1.0",
    )
    .server("http://localhost:3000/api");

    let ui = api_service.swagger_ui();

    let app = Route::new()
        .nest("/api", api_service)
        .nest("/docs", ui)
        .with(Cors::new());

    println!("Server is running on http://localhost:3000");
    println!("API documentation available at http://localhost:3000/docs");

    Server::new(TcpListener::bind("0.0.0.0:3000"))
        .run(app)
        .await
}
