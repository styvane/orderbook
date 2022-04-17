use std::str::FromStr;

use once_cell::sync::Lazy;
use orderbook::prelude::{OrderBookServer, SummaryService};
use orderbook::telemetry::Tracer;
use tonic::transport::Server;
use tracing::Level;

static TRACER: Lazy<()> = Lazy::new(|| match std::env::var("RUST_LOG") {
    Ok(value) if Level::from_str(&value).is_ok() => {
        Tracer::new("orderbook", &value).init(std::io::stdout);
    }
    _ => (),
});

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    Lazy::force(&TRACER);

    let summary = SummaryService::new();
    let addr = summary.config.server_addr().parse().unwrap();
    let server = OrderBookServer::new(summary);
    println!("Starting server at http://{}", addr);
    Server::builder().add_service(server).serve(addr).await?;
    Ok(())
}
