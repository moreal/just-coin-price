use poem::EndpointExt;
use poem::middleware::Cors;
use poem::{Route, Server, listener::TcpListener};
use poem_openapi::OpenApiService;

mod api;
mod vendors;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    if std::env::var_os("RUST_LOG").is_none() {
        unsafe {
            std::env::set_var("RUST_LOG", "poem=debug");
        }
    }

    let allowed_tickers: Vec<String> = std::env::var_os("ALLOWED_TICKERS")
        .map(|s| s.into_string().unwrap_or_default())
        .map(|s| s.split(',').map(|x| x.to_string()).collect())
        .unwrap_or_default();
    let allowed_currencies: Vec<String> = std::env::var_os("ALLOWED_CURRENCIES")
        .map(|s| s.into_string().unwrap_or_default())
        .map(|s| s.split(',').map(|x| x.to_string()).collect())
        .unwrap_or_default();

    let cmc_api_key = std::env::var("CMC_API_KEY")
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::NotFound, e))?;

    tracing_subscriber::fmt::init();

    let api_service = OpenApiService::new(
        api::Api {
            vendor: Box::new(vendors::CacheLayerVendor::new(Box::new(
                vendors::CoinMarketCapVendor::new(cmc_api_key),
            ))),
            allowed_tickers,
            allowed_currencies,
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
