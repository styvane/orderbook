//! Bitstamp integration.
//!
//! This module implements the integration with Bitstamp.

use async_trait::async_trait;
use futures_util::SinkExt;
use tokio_tungstenite::connect_async;
use tungstenite::Message;

use super::transport::WebSocketTransport;
use super::transport::{StopSender, WebSocketStream};
use crate::configuration::Configuration;
use crate::configuration::ExchangeConfig;
use crate::prelude::{Error, Exchange};

pub struct ExchangeApi {
    config: Configuration,
    socket: WebSocketStream,
    send_on_stop: StopSender,
}

impl ExchangeApi {
    /// Opens a co nnection to an exchange.
    #[tracing::instrument(name = "Connect to websocket", skip(self))]
    async fn connect(&mut self, address: &str) -> Result<(), Error> {
        let (socket, _) = connect_async(address).await.map_err(Error::WsError)?;
        self.socket = Box::pin(socket) as WebSocketStream;
        Ok(())
    }
}

#[async_trait]
impl WebSocketTransport for ExchangeApi {
    #[tracing::instrument(name = "Subscribe to channel", skip(self))]
    async fn subscribe(&mut self, message: Message) -> Result<(), Error> {
        self.socket.send(message).await?;
        Ok(())
    }

    async fn unsubscribe(&self) -> Result<(), Error> {
        Ok(())
    }
}

#[tracing::instrument(name = "Create new subscribe message", skip(config))]
pub fn subscribe_message(config: &ExchangeConfig) -> Result<Message, Error> {
    let exchange = config.exchange.parse()?;
    let message = match exchange {
        Exchange::Bitstamp => Message::Text(
            serde_json::json!({
                "event": "bts:subscribe",
                "data": {
                    "channel": format!("order_book_{}", &config.channel)
                }
            })
            .to_string(),
        ),
        Exchange::Binance => Message::Text(
            serde_json::json!({
                "method": "SUBSCRIBE",
                "params": [format!("{}@depth", &config.channel)],
                "id": 1
            })
            .to_string(),
        ),
    };

    Ok(message)
}
