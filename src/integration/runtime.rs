use std::process;

use tokio::sync::{mpsc, oneshot};

use super::api_service::ApiService;
use super::transport::WebSocketTransport;
use crate::configuration::Configuration;
use crate::prelude::{Book, BookKind};

#[tracing::instrument(name = "Run until stopped", skip(book_sender, stop_publisher))]
pub async fn run_until_stopped(
    book_sender: mpsc::Sender<(BookKind, Book)>,
    stop_publisher: oneshot::Receiver<bool>,
) {
    let config = match Configuration::new() {
        Ok(config) => config,
        Err(e) => {
            tracing::error!("failed to initialize configuration: {}", e);
            process::exit(1);
        }
    };

    let mut api = match ApiService::new() {
        Ok(api) => api,
        Err(e) => {
            tracing::error!("failed to create api {}", e);
            process::exit(1);
        }
    };

    for config in &config.exchanges {
        if let Err(e) = api.connect(config).await {
            tracing::error!("connection to exchange '{}' failed: {}", config.exchange, e);
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
