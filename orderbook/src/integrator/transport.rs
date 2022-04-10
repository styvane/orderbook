//! Transport trait.
//!
//! The transport traits encapsulate the operations required of a transport mechanism.

use async_trait::async_trait;
use tokio::sync::mpsc::Sender;

use crate::error::Error;
use crate::order_book::Book;

#[async_trait]
pub trait WebSocketTransport {
    /// Opens a connection to an exchange.
    async fn connect(&mut self) -> Result<(), Error>;

    /// Subscribes to a channel.
    /// This method should only be called when a connection was previously
    /// successfully established.
    async fn subscribe(&mut self, channel: &str) -> Result<(), Error>;

    /// Reads an incoming stream for a previously subscribed channel.
    /// It creates a book and send the result back specified sender.
    async fn watch(&mut self, sender: Sender<Book>) -> Result<(), Error>;

    /// Unsubscribes from a channel.
    async fn unsubscribe(&self) -> Result<(), Error>;
}
