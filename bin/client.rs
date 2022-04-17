use orderbook::prelude::{Empty, OrderBookClient};
use tonic::Request;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = OrderBookClient::connect("http://[::1]:12000").await?;

    let mut stream = client
        .book_summary(Request::new(Empty {}))
        .await?
        .into_inner();

    while let Some(summary) = stream.message().await? {
        println!("{summary:?}");
    }
    Ok(())
}
