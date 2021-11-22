use std::net::{SocketAddr};
use std::str::FromStr;

static DEFAULT_ADDR: &str = "127.0.0.1";

#[tokio::main]
async fn main() {
    let addr = std::env::var("ADDR").unwrap_or_else(|_| String::from(DEFAULT_ADDR));
    let addr = SocketAddr::from_str(&addr).expect("Invalid address");
    peakmusic::server::start(addr).await;
}
