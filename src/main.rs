mod demo;
mod display;
mod monitor;
mod system;
mod zfs;

use std::env;
use std::process;

fn main() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async_main());
}

async fn async_main() {
    let args: Vec<String> = env::args().collect();

    // Parse command line arguments: [POOL] [INTERVAL]
    let pool = args.get(1).map(|s| s.as_str());
    let interval = args.get(2).and_then(|s| s.parse::<u32>().ok()).unwrap_or(2); // Default 2 seconds

    // Check for demo mode
    let demo_mode = env::var("DEMO_MODE").unwrap_or_else(|_| "false".to_string()) == "true";

    if let Err(e) = monitor::run_with_args(demo_mode, pool, interval).await {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}
