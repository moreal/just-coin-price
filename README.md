# just-coin-price

A simple API server to retrieve coin prices from multiple vendors in a consistent API interface.

## Building

To build the project:

```bash
cargo build
```

For a release build:

```bash
cargo build --release
```

## Running

To run the application:

```bash
cargo run
```

## Testing

To run the tests:

```bash
cargo test
```

## API Endpoints

- `GET /coins/{ticker}/price?currency={currency}` - Get the latest price for a specific cryptocurrency for a specific currency.

## Environment Variables

- `ALLOWED_TICKERS`: A comma-separated list of tickers to allow. (e.g., `ALLOWED_TICKERS=WNCG`)
- `ALLOWED_CURRENCIES`: A comma-separated list of currencies to allow. (e.g., `ALLOWED_CURRENCIES=KRW,USD`)

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
