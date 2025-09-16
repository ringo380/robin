mod engine;
mod examples;

use examples::{run_basic_window, run_magical_demo};

#[tokio::main]
async fn main() {
    // Choose which demo to run
    let demo = std::env::args().nth(1).unwrap_or_else(|| "magical".to_string());
    
    match demo.as_str() {
        "basic" => run_basic_window().await,
        "magical" => run_magical_demo().await,
        _ => {
            println!("Available demos:");
            println!("  cargo run basic    - Basic windowed demo");
            println!("  cargo run magical  - Magical effects demo (default)");
            run_magical_demo().await;
        }
    }
}
