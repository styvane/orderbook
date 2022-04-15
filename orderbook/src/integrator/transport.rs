//! Transport type
//!
//! This module defines various data structures and mechanism for transport.

use std::pin::Pin;

use crate::error::Error;
use async_trait::async_trait;
use futures_util::{Sink, Stream};
use tokio::sync::oneshot;
use tungstenite::Message;

/// The [`StopSender`] type wraps an optional tokio::oneshot sender end.
pub struct StopSender(Option<oneshot::Sender<bool>>);

impl StopSender {
    /// Creates new stop [`StopSender`]
    pub fn new(sender: oneshot::Sender<bool>) -> Self {
        Self(Some(sender))
    }

    /// Requests stop operation by sending a message.
    pub fn try_stop(&mut self) -> Result<(), Error> {
        self.0
            .take()
            .ok_or_else(|| anyhow::anyhow!("channel already used or missing"))?
            .send(true)
            .map_err(|_| anyhow::anyhow!("failed to stop transport"))?;

        Ok(())
    }
}

/// The [`SocketStream`] trait encapsulates the socket IO mechanism.
pub trait SocketStream<I, E>: Stream<Item = Result<I, E>> + Sink<I, Error = E> {}

impl<T, I, E> SocketStream<I, E> for T where T: Stream<Item = Result<I, E>> + Sink<I, Error = E> {}

/// The [`WebSocketStream`] type is a pinned and boxed `SocketStream`.
pub type WebSocketStream =
    Pin<Box<dyn SocketStream<Message, tungstenite::Error> + Send + Sync + 'static>>;

///The transport traits encapsulate the operations required of a transport mechanism.
#[async_trait]
pub trait WebSocketTransport {
    /// Subscribes to a channel.
    /// This method should only be called when a connection was previously
    /// successfully established.
    async fn subscribe(&mut self, channel: &str) -> Result<(), Error>;

    /// Unsubscribes from a channel.
    async fn unsubscribe(&self) -> Result<(), Error>;
}
