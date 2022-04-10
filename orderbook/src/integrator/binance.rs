//! Binance integration.
//!
//! This module implements the integration with Binance.

use anyhow::Context;
use async_trait::async_trait;
use futures_util::{SinkExt, StreamExt};
use tokio::net::TcpStream;
use tokio::sync::mpsc::Sender;
use tokio_tungstenite::{connect_async, MaybeTlsStream, WebSocketStream};
use tungstenite::Message;

use super::event_data::EventText;
use super::transport::WebSocketTransport;
use crate::error::Error;
use crate::order_book::Book;

pub struct Binance {
    address: String,
    socket: WebSocketStream<MaybeTlsStream<TcpStream>>,
}

#[derive(Debug, serde::Deserialize)]
pub enum Event {
    Subscribe { result: Option<String>, id: usize },
    OrderBook(EventText),
}

#[async_trait]
impl WebSocketTransport for Binance {
    /// Opens a connection to an exchange.
    async fn connect(&mut self) -> Result<(), Error> {
        let (socket, _) = connect_async(&self.address).await.map_err(Error::WsError)?;
        self.socket = socket;
        Ok(())
    }
    async fn subscribe(&mut self, channel: &str) -> Result<(), Error> {
        self.socket
            .send(Message::Text(
                serde_json::json!({
                    "event": "bts:subscribe",
                    "data": {
                        "channel": format!("order_book_{channel}")
                    }
                })
                .to_string(),
            ))
            .await?;
        Ok(())
    }
    async fn watch(&mut self, sender: Sender<Book>) -> Result<(), Error> {
        while let Some(message) = self.socket.next().await {
            let message = message?;
            let event: Event = serde_json::from_str(&message.into_text()?)?;
            match event {
                Event::OrderBook(event) => {
                    for (price, amount) in event.bids {
                        sender
                            .send(Book::new(price, amount))
                            .await
                            .context("failed to send book")?;
                    }
                }
                Event::Subscribe { .. } => {
                    todo!()
                }
            }
        }

        Ok(())
    }
    async fn unsubscribe(&self) -> Result<(), Error> {
        Ok(())
    }
}
