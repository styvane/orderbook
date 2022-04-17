//! API integration.
//!
//! This module implements the general API integration operations.

use async_trait::async_trait;
use futures_util::{SinkExt, StreamExt};
use std::collections::HashMap;
use tokio::sync::{mpsc, oneshot};
use tokio_tungstenite::connect_async;
use tungstenite::Message;

use super::transport::WebSocketTransport;
use super::transport::{StopSender, WebSocketStream};
use crate::configuration::ExchangeConfig;
use crate::prelude::{Book, BookKind, Error, Exchange, Result};

const DEFAULT_RESULT_SIZE: usize = 10;

pub struct ApiService {
    pub result_size: usize,
    pub(crate) services: Vec<ExchangeService>,
    pub send_on_stop: HashMap<String, StopSender>,
}

impl ApiService {
    /// Creates new [`ApiService`].
    pub fn new() -> Result<Self> {
        Ok(Self {
            result_size: DEFAULT_RESULT_SIZE,
            services: vec![],
            send_on_stop: HashMap::new(),
        })
    }

    /// Opens a connection to an exchange.
    #[tracing::instrument(name = "Connect to websocket", skip(self, config))]
    pub async fn connect(&mut self, config: &ExchangeConfig) -> Result<()> {
        let (socket, _) = connect_async(&config.url).await.map_err(Error::WsError)?;
        let socket = Box::pin(socket) as WebSocketStream;
        self.services.push(ExchangeService {
            socket: Some(socket),
            config: config.clone(),
        });
        Ok(())
    }

    #[tracing::instrument(name = "Watch list of socket stream", skip(self, book_sender, stop))]
    pub async fn watch(
        &mut self,
        book_sender: mpsc::Sender<(BookKind, Book)>,
        mut stop: oneshot::Receiver<bool>,
    ) {
        let mut fut = futures_util::stream::select_all(
            self.services.iter_mut().map(|s| s.socket.take().unwrap()),
        );

        loop {
            tokio::select! {
                Some(message) = fut.next() => {
                    tracing::info!("{:?}", message);
                    tokio::spawn(Book::publish(book_sender.clone(), message));

                },
                _ = (&mut stop) => break,
            }
        }

        // while let Some(message) = fut.next().await {
        //     tokio::spawn(Book::publish(book_sender.clone(), message));
        // }
    }
}

/// Exchange service.
pub struct ExchangeService {
    pub socket: Option<WebSocketStream>,
    pub config: ExchangeConfig,
}

#[async_trait]
impl WebSocketTransport for ExchangeService {
    #[tracing::instrument(name = "Subscribe to channel", skip(self, message))]
    async fn subscribe(&mut self, message: Message) -> Result<()> {
        self.socket.as_mut().unwrap().send(message).await?;
        Ok(())
    }

    async fn unsubscribe(&self) -> Result<()> {
        Ok(())
    }
}

impl ExchangeService {
    /// Creates new message for based on exchange configuration.
    #[tracing::instrument(name = "Create new subscribe message", skip(self))]
    pub fn new_message(&self) -> Result<Message> {
        let exchange = self.config.exchange.parse()?;
        let message = match exchange {
            Exchange::Bitstamp => Message::Text(
                serde_json::json!({
                    "event": "bts:subscribe",
                    "data": {
                        "channel": format!("order_book_{}", &self.config.channel)
                    }
                })
                .to_string(),
            ),
            Exchange::Binance => Message::Text(
                serde_json::json!({
                    "method": "SUBSCRIBE",
                    "params": [format!("{}@depth", &self.config.channel)],
                    "id": 1
                })
                .to_string(),
            ),
        };

        Ok(message)
    }
}
