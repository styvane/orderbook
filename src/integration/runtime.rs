use std::process;

use tokio::sync::{mpsc, oneshot};

use super::api_service::ApiService;
use super::transport::WebSocketTransport;
use crate::configuration::ExchangeConfig;
use crate::prelude::{Book, BookKind};

#[tracing::instrument(name = "Run until stopped", skip(book_sender, stop_publisher, config))]
pub async fn run_until_stopped(
    capacity: usize,
    config: Vec<ExchangeConfig>,
    book_sender: mpsc::Sender<(BookKind, Book)>,
    stop_publisher: oneshot::Receiver<bool>,
) {
    let mut api = ApiService::new(capacity);

    for val in &config {
        if let Err(e) = api.connect(val).await {
            tracing::error!("connection to exchange '{}' failed: {}", val.exchange, e);
        }
    }

    if api.services.is_empty() {
        tracing::error!("no connection was established");
        process::exit(1);
    }

    for service in api.services.iter_mut() {
        match service.new_message() {
            Ok(message) => {
                if let Err(e) = service.subscribe(message).await {
                    tracing::error!(
                        "failed to subscribe to exchange '{}', {}",
                        &service.config.exchange,
                        e
                    );
                }
            }
            Err(e) => {
                tracing::error!(
                    "failed to create subscription message for exchange '{}': {}",
                    &service.config.exchange,
                    e
                );
                process::exit(1);
            }
        }
    }

    api.watch(book_sender, stop_publisher).await;
}
