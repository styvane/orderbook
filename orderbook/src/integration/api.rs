//! Bitstamp integration.
//!
//! This module implements the integration with Bitstamp.

use async_trait::async_trait;
use futures_util::SinkExt;
use std::collections::HashMap;
use tokio_tungstenite::connect_async;
use tungstenite::Message;

use super::transport::WebSocketTransport;
use super::transport::{StopSender, WebSocketStream};
use crate::configuration::Configuration;
use crate::configuration::ExchangeConfig;
use crate::prelude::{Error, Exchange, Result};

pub struct ExchangeApi {
    config: Configuration,
    sockets: Option<Vec<WebSocketStream>>,
    send_on_stop: HashMap<String, StopSender>,
}

impl ExchangeApi {
    /// Opens a co nnection to an exchange.
    #[tracing::instrument(name = "Connect to websocket", skip(self))]
    pub async fn connect(&mut self, address: &str) -> Result<()> {
        let (socket, _) = connect_async(address).await.map_err(Error::WsError)?;

        if let Some(ref mut streams) = self.sockets {
            streams.push(Box::pin(socket) as WebSocketStream);
        } else {
            let sockets = vec![Box::pin(socket) as WebSocketStream];
            self.sockets = Some(sockets)
        }

        Ok(())
    }
}

#[async_trait]
impl WebSocketTransport for ExchangeApi {
    #[tracing::instrument(name = "Subscribe to channel", skip(self))]
    async fn subscribe(&mut self, message: Message) -> Result<()> {
        self.socket.send(message).await?;
        Ok(())
    }

    async fn unsubscribe(&self) -> Result<()> {
        Ok(())
    }
}

#[tracing::instrument(name = "Create new subscribe message", skip(config))]
pub fn subscribe_message(config: &ExchangeConfig) -> Result<Message> {
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
