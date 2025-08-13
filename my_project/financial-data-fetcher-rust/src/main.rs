use chrono::Utc;
use serde::Deserialize;
use std::fs::{File, OpenOptions};
use std::io::{self, Write};
use std::path::Path;
use std::thread;
use std::time::Duration;
use thiserror::Error;
use serde::de::Error as _;

// ---------- Error Type ----------
#[derive(Debug, Error)]
enum FetchError {
    #[error("HTTP error: {0}")]
    Http(String),
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("CSV parse error: {0}")]
    Csv(String),
    #[error("Env var missing: {0}")]
    Env(String),
}

impl From<ureq::Error> for FetchError {
    fn from(e: ureq::Error) -> Self {
        FetchError::Http(e.to_string())
    }
}

// ---------- Pricing Trait ----------
trait Pricing {
    fn name(&self) -> &str;
    fn fetch_price(&self) -> Result<f64, FetchError>;
    fn file_path(&self) -> &str;
    fn save_to_file(&self, price: f64) -> Result<(), FetchError> {
        let path = self.file_path();
        let file_exists = Path::new(path).exists();

        // Open for append; create if missing
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)?;

        // Write header once
        if !file_exists || std::fs::metadata(path)?.len() == 0 {
            writeln!(file, "timestamp_utc,price")?;
        }

        let ts = Utc::now().to_rfc3339();
        writeln!(file, "{},{}", ts, price)?;
        Ok(())
    }
}

// ---------- CoinGecko (BTC & ETH) ----------
#[derive(Debug, Clone)]
struct Bitcoin;
#[derive(Debug, Clone)]
struct Ethereum;

#[derive(Debug, Deserialize)]
struct CoinSimpleResp {
    usd: f64,
}

fn fetch_coingecko_usd(coin_id: &str) -> Result<f64, FetchError> {
    // Public endpoint, no API key required
    let url = format!(
        "https://api.coingecko.com/api/v3/simple/price?ids={}&vs_currencies=usd",
        coin_id
    );
    let resp = ureq::get(&url)
        .set("User-Agent", "financial-data-fetcher/0.1")
        .call()?;
    let json: serde_json::Value = resp.into_json()?;
    let price = json
        .get(coin_id)
        .and_then(|m| m.get("usd"))
        .and_then(|v| v.as_f64())
        .ok_or_else(|| FetchError::Json(serde_json::Error::custom("missing usd price")))?;
    Ok(price)
}

impl Pricing for Bitcoin {
    fn name(&self) -> &str { "Bitcoin" }
    fn fetch_price(&self) -> Result<f64, FetchError> {
        fetch_coingecko_usd("bitcoin")
    }
    fn file_path(&self) -> &str { "data/bitcoin.csv" }
}

impl Pricing for Ethereum {
    fn name(&self) -> &str { "Ethereum" }
    fn fetch_price(&self) -> Result<f64, FetchError> {
        fetch_coingecko_usd("ethereum")
    }
    fn file_path(&self) -> &str { "data/ethereum.csv" }
}

// ---------- S&P 500 proxy (SPY) ----------
#[derive(Debug, Clone)]
struct SP500; // logical "S&P 500" asset

#[derive(Debug, Deserialize)]
struct AvGlobalQuote {
    #[serde(rename = "Global Quote")]
    global: Option<AvQuoteFields>,
}

#[derive(Debug, Deserialize)]
struct AvQuoteFields {
    #[serde(rename = "05. price")]
    price: Option<String>,
}

fn fetch_spy_alpha_vantage() -> Result<f64, FetchError> {
    let key = std::env::var("ALPHA_VANTAGE_KEY")
        .map_err(|_| FetchError::Env("ALPHA_VANTAGE_KEY".into()))?;
    let url = format!("https://www.alphavantage.co/query?function=GLOBAL_QUOTE&symbol=SPY&apikey={}", key);
    let resp = ureq::get(&url)
        .set("User-Agent", "financial-data-fetcher/0.1")
        .call()?;
    let parsed: AvGlobalQuote = resp.into_json()?;
    let price_str = parsed
        .global
        .and_then(|g| g.price)
        .ok_or_else(|| FetchError::Json(serde_json::Error::custom("missing price in AV response")))?;
    let price = price_str
        .parse::<f64>()
        .map_err(|e| FetchError::Csv(format!("parse float: {}", e)))?;
    Ok(price)
}

fn fetch_spy_stooq() -> Result<f64, FetchError> {
    // Stooq quick quote CSV for SPY
    let url = "https://stooq.com/q/l/?s=spy&f=sd2t2ohlcvn";
    let body = ureq::get(url)
        .set("User-Agent", "financial-data-fetcher/0.1")
        .call()?
        .into_string()?;
    // Take the second line, split by comma, take Close column (index 6)
    let mut lines = body.lines();
    let _ = lines.next(); // header
    if let Some(line) = lines.next() {
        let cols: Vec<&str> = line.split(',').collect();
        if cols.len() >= 7 {
            let close = cols[6].trim().parse::<f64>()
                .map_err(|e| FetchError::Csv(format!("stooq parse float: {}", e)))?;
            return Ok(close);
        }
    }
    Err(FetchError::Csv("unexpected Stooq CSV format".into()))
}

impl Pricing for SP500 {
    fn name(&self) -> &str { "S&P 500 (SPY proxy)" }
    fn fetch_price(&self) -> Result<f64, FetchError> {
        match fetch_spy_alpha_vantage() {
            Ok(p) => Ok(p),
            Err(e) => {
                eprintln!("[SP500] Alpha Vantage failed ({e}). Falling back to Stooq...");
                fetch_spy_stooq()
            }
        }
    }
    fn file_path(&self) -> &str { "data/sp500.csv" }
}

// ---------- Main ----------
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Ensure data directory exists
    std::fs::create_dir_all("data")?;

    // Assets vector using trait objects
    let assets: Vec<Box<dyn Pricing>> = vec![
        Box::new(Bitcoin),
        Box::new(Ethereum),
        Box::new(SP500),
    ];

    println!("Starting Financial Data Fetcher (10s interval). Press Ctrl+C to stop.");
    loop {
        for asset in &assets {
            match asset.fetch_price() {
                Ok(price) => {
                    if let Err(e) = asset.save_to_file(price) {
                        eprintln!("[{}] Failed to save: {}", asset.name(), e);
                    } else {
                        println!("[{}] {} -> saved", asset.name(), price);
                    }
                }
                Err(e) => eprintln!("[{}] Fetch error: {}", asset.name(), e),
            }
        }
        thread::sleep(Duration::from_secs(10));
    }
}