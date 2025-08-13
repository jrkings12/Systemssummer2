# Financial Data Fetcher (Rust)

Periodically fetches and records pricing data for **Bitcoin**, **Ethereum**, and the **S&P 500** (via SPY proxy).  
Data is appended to CSV files in `data/` every **10 seconds**.

## Features
- Three structs: `Bitcoin`, `Ethereum`, `SP500`.
- `Pricing` trait implements a common interface: `fetch_price()` and file persistence.
- HTTP via `ureq`, JSON parsing via `serde/serde_json`, timestamping via `chrono`.
- BTC/ETH from **CoinGecko** public API.
- S&P 500 via **Alpha Vantage** (if `ALPHA_VANTAGE_KEY` is set), otherwise falls back to **Stooq** CSV for SPY.
- Robust error handling with `thiserror` and fallbacks.

## Quickstart

```bash
# 1) Ensure Rust toolchain
rustc --version
cargo --version

# 2) (Optional) Set Alpha Vantage API key for best SPY price:
#    https://www.alphavantage.co/support/#api-key
export ALPHA_VANTAGE_KEY="YOUR_KEY_HERE"

# 3) Run
cargo run --release
```

You should see logs like:

```
Starting Financial Data Fetcher (10s interval). Press Ctrl+C to stop.
[Bitcoin] 67890.12 -> saved
[Ethereum] 3456.78 -> saved
[S&P 500 (SPY proxy)] 556.12 -> saved
```

CSV files will be created (with headers) at:
- `data/bitcoin.csv`
- `data/ethereum.csv`
- `data/sp500.csv`

Each line is `timestamp_utc,price` (RFC 3339 timestamp).

## Design Notes

- **Trait-based** polymorphism lets you add more assets by creating a new struct and implementing `Pricing`.
- **Error handling** bubbles up precise error types and preserves context.
- **Alpha Vantage vs Stooq**: SPY is used as a real-time proxy for the S&P 500 index. If `ALPHA_VANTAGE_KEY` is
  absent or rate-limited, we fall back to the free Stooq CSV quick quote for SPY's last trade/close.

## GitHub Setup

To publish this project:

```bash
# inside this folder
git init
git add .
git commit -m "Initial commit: financial data fetcher"
gh repo create your-username/financial-data-fetcher --public --source=. --remote=origin --push
# or manually:
# git remote add origin https://github.com/your-username/financial-data-fetcher.git
# git push -u origin main
```

Then share your repository link.

## Screenshot Instructions

1. Run the app: `cargo run --release`
2. Keep the terminal visible; press `PrtSc` (Windows) or `Cmd+Shift+4` (macOS) to capture.
3. Save and upload the screenshot to your repo (e.g., `assets/screenshot.png`) and reference it in this README.

## Notes & Limits

- Free providers may throttle or rate-limit; errors will be logged and the loop will continue.
- This code writes headers if the file doesn't exist or is empty.
- The loop runs indefinitely until you press `Ctrl+C`.

## License
MIT