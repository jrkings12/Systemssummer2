use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};
use chrono::Utc;
use serde::Serialize;
use serde_json;
use std::env;

// struct for output
#[derive(Serialize, Debug)]
struct WebsiteStatus {
    url: String,
    status: Result<u16, String>,
    response_time_ms: u128,
    timestamp: chrono::DateTime<Utc>,
}

// function to check a single website with retries
fn check_website(url: &str, timeout_secs: u64, max_retries: u32) -> WebsiteStatus {
    let mut attempts = 0;
    let start = Instant::now();
    let now = Utc::now();

    while attempts <= max_retries {
        let agent = ureq::AgentBuilder::new()
            .timeout(Duration::from_secs(timeout_secs))
            .build();

        match agent.get(url).call() {
            Ok(resp) => {
                return WebsiteStatus {
                    url: url.to_string(),
                    status: Ok(resp.status()),
                    response_time_ms: start.elapsed().as_millis(),
                    timestamp: now,
                };
            }
            Err(e) => {
                attempts += 1;
                if attempts > max_retries {
                    return WebsiteStatus {
                        url: url.to_string(),
                        status: Err(format!("Error after {} retries: {}", max_retries, e)),
                        response_time_ms: start.elapsed().as_millis(),
                        timestamp: now,
                    };
                }
            }
        }
    }

    // should never reach here
    WebsiteStatus {
        url: url.to_string(),
        status: Err("Unknown error".to_string()),
        response_time_ms: start.elapsed().as_millis(),
        timestamp: now,
    }
}

fn main() {
    // command-line arguments
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: cargo run -- <url1> <url2> ... [--workers=N] [--timeout=S] [--retries=R]");
        return;
    }

    // default configuration
    let mut urls = Vec::new();
    let mut workers = 10;
    let mut timeout = 5;
    let mut max_retries = 2;

    // parse arguments
    for arg in &args[1..] {
        if arg.starts_with("--workers=") {
            workers = arg["--workers=".len()..].parse().unwrap_or(workers);
        } else if arg.starts_with("--timeout=") {
            timeout = arg["--timeout=".len()..].parse().unwrap_or(timeout);
        } else if arg.starts_with("--retries=") {
            max_retries = arg["--retries=".len()..].parse().unwrap_or(max_retries);
        } else {
            urls.push(arg.clone());
        }
    }

    let (tx, rx) = mpsc::channel();

    // worker threads
    let url_chunks = urls.chunks((urls.len() + workers - 1) / workers);
    for chunk in url_chunks {
        let tx_clone = tx.clone();
        let chunk_vec = chunk.to_vec();
        thread::spawn(move || {
            for url in chunk_vec {
                let result = check_website(&url, timeout, max_retries);
                tx_clone.send(result).unwrap();
            }
        });
    }

    drop(tx); // Close sender

    // Receive and print results as JSON
    for received in rx {
        println!("{}", serde_json::to_string_pretty(&received).unwrap());
    }

    println!("Finished checking {} websites with {} workers.", urls.len(), workers);
}
