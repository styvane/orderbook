//! Binance integration.
//!
//! This module implements the integration with Binance.

use async_trait::async_trait;
use futures_util::SinkExt;
use tokio_tungstenite::connect_async;
use tungstenite::Message;

use super::transport::WebSocketTransport;
use super::transport::{StopSender, WebSocketStream};
use crate::error::Error;
pub struct Binance {
    address: String,
    socket: WebSocketStream,
    send_on_stop: StopSender,
}

impl Binance {
    /// Opens a co nnection to an exchange.
    async fn connect(&mut self) -> Result<(), Error> {
        let (socket, _) = connect_async(&self.address).await.map_err(Error::WsError)?;
        self.socket = Box::pin(socket) as WebSocketStream;
        Ok(())
    }
}

#[async_trait]
impl WebSocketTransport for Binance {
    async fn subscribe(&mut self, currency_pair: &str) -> Result<(), Error> {
        self.socket
            .send(Message::Text(
                serde_json::json!({
                    "method": "SUBSCRIBE",
                    "params": [format!("{currency_pair}@depth")],
                    "id": 1
                })
                .to_string(),
            ))
            .await?;
        Ok(())
    }

    async fn unsubscribe(&self) -> Result<(), Error> {
        Ok(())
    }
}
