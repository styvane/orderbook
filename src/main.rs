use once_cell::sync::Lazy;
use orderbook::prelude::{OrderBookServer, SummaryService};
use orderbook::telemetry::Tracer;
use tonic::transport::Server;

static TRACER: Lazy<()> = Lazy::new(|| {
    if std::env::var("TEST_LOG").is_ok() {
        Tracer::new("orderbook", "debug").init(std::io::stdout);
    }
});

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    Lazy::force(&TRACER);
    let addr = "[::1]:10000".parse().unwrap();

    let summary = SummaryService;
    let server = OrderBookServer::new(summary);

    Server::builder().add_service(server).serve(addr).await?;
    Ok(())
}
