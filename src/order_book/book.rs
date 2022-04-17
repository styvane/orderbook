//! Order book type.
//!
//! This module defines the data structures for maintaining an order book.

use std::cmp::Ordering;

use priority_queue::DoublePriorityQueue;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;

use crate::integration::event::{Event, EventData};
use crate::prelude::{Error, Exchange};

tonic::include_proto!("orderbook");

/// The [`BookQueue`] type. the [See module level documentation](self).
#[derive(Debug)]
pub struct BookQueue {
    cap: usize,
    pub(super) kind: BookKind,
    pub(super) books: DoublePriorityQueue<Exchange, Book>,
}

impl PartialOrd for Book {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Eq for Book {}

impl Ord for Book {
    fn cmp(&self, other: &Self) -> Ordering {
        let amount: Decimal = self.amount.parse().unwrap();
        let price: Decimal = self.price.parse().unwrap();

        let rh_amount: Decimal = other.amount.parse().unwrap();
        let rh_price: Decimal = other.price.parse().unwrap();

        match amount.cmp(&rh_amount) {
            Ordering::Equal => price.cmp(&rh_price),
            Ordering::Greater => Ordering::Less,
            Ordering::Less => Ordering::Greater,
        }
    }
}

/// The [`BookKind`] type is the different kind of books in an order book.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all(serialize = "snake_case"))]
pub enum BookKind {
    Asks,
    Bids,
}

impl AsRef<str> for BookKind {
    fn as_ref(&self) -> &str {
        match self {
            BookKind::Asks => "ASKS",
            BookKind::Bids => "BIDS",
        }
    }
}

impl Book {
    /// Creates new book with the specified price, amount and exchange.
    pub fn new(price: &str, amount: &str, exchange: &str) -> Self {
        Book {
            price: price.into(),
            amount: amount.into(),
            exchange: exchange.into(),
        }
    }

    /// Publishes books to a channel.
    #[tracing::instrument(name = "Publishes books to a channel", skip(book_sender, messages))]
    pub async fn publish(
        book_sender: mpsc::Sender<(BookKind, Book)>,
        messages: Result<tungstenite::Message, tungstenite::Error>,
    ) -> Result<(), Error> {
        let messages = messages.unwrap();
        let exchange: Exchange;
        let EventData { bids, asks } = match serde_json::from_str::<Event>(&messages.into_text()?)
            .map_err(Error::ParseError)
        {
            Ok(Event::Binance(event)) => {
                exchange = Exchange::Binance;
                event
            }
            Ok(Event::Bitstamp { data }) => {
                exchange = Exchange::Bitstamp;
                data
            }
            Err(e) => {
                tracing::error!("failed to parse message: {}", e);
                return Err(e);
            }
        };

        let bid_sender = book_sender.clone();
        let bid_exchange = exchange.clone();

        tokio::spawn(async move {
            for (price, amount) in bids {
                let book = Book::new(&price, &amount, bid_exchange.as_ref());
                if let Err(e) = bid_sender.send((BookKind::Bids, book)).await {
                    tracing::error!("failed to publish book: {}", e);
                }
            }
        });

        tokio::spawn(async move {
            for (price, amount) in asks {
                let book = Book::new(&price, &amount, exchange.as_ref());
                if let Err(e) = book_sender.send((BookKind::Asks, book)).await {
                    tracing::error!("failed to publish book: {}", e);
                }
            }
        });

        Ok(())
    }
}

impl BookQueue {
    /// Creates new order book of the specified kind.
    pub fn new(kind: BookKind) -> Self {
        BookQueue {
            cap: 0,
            kind,
            books: DoublePriorityQueue::new(),
        }
    }

    /// Creates new order book of the specified kind with the specified capacity.
    pub fn with_capacity(kind: BookKind, capacity: usize) -> Self {
        BookQueue {
            cap: capacity,
            kind,
            books: DoublePriorityQueue::with_capacity(capacity),
        }
    }

    /// Adds a book to the order book.
    ///
    /// # Example
    ///
    ///
    ///
    /// use orderbook::prelude::*;
    ///
    /// let mut order_book = BookQueue::with_capacity(BookKind::Asks, 1);
    /// let book = Book::new("2.1", "0.4", "bitstamp");
    /// order_book.push(Exchange::Bitstamp, book);
    /// assert_eq!(order_book.len(), 1);
    /// ```
    pub fn push(&mut self, exchange: Exchange, book: Book) {
        self.books.push(exchange, book);
    }

    /// Removes the minimum book from the order book.
    ///
    /// # Example
    ///
    ///
    /// use orderbook::prelude::*;
    ///
    /// let mut order_book = BookQueue::with_capacity(BookKind::Bids, 1);
    /// order_book.push(Exchange::Binance, Book::new("1.9", "3.7', "binance"));
    /// order_book.push(Exchange::Bitstamp, Book::new("2.5", "4.1", "bitstamp"));
    /// let value = Some((Exchange::Binance, Book::new("1.9", "3.7", "binance")));
    /// assert_eq!(order_book.pop(), value);
    /// ```
    pub fn pop(&mut self) -> Option<(Exchange, Book)> {
        if self.len() == self.cap {
            return self.books.pop_min();
        }
        None
    }

    /// Returns the number of books in the order book.
    pub fn len(&self) -> usize {
        self.books.len()
    }

    /// Returns `true` if the book order has a lenght of 0.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns the top n books.
    pub fn take(&self, n: usize) -> Vec<Book> {
        self.books
            .clone()
            .into_sorted_iter()
            .rev()
            .take(n)
            .map(|(_, b)| b)
            .collect::<Vec<_>>()
    }

    /// Returns the max price.
    pub fn max_price(&self) -> Decimal {
        self.books
            .peek_max()
            .map(|(_, b)| b.price.parse::<Decimal>().unwrap_or_default())
            .unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::Book;
    use crate::telemetry::tests::force_lazy;
    use fake;
    use tokio::sync::mpsc::channel;
    use tungstenite::Message;

    fn generate_message_data(ask: &str, bid: &str) -> String {
        force_lazy();
        let gen = || -> Vec<(String, String)> {
            fake::vec![f64; 10]
                .iter()
                .map(ToString::to_string)
                .zip(fake::vec![f64; 10].iter().map(ToString::to_string))
                .collect()
        };

        serde_json::json!({
            ask: gen(),
            bid: gen(),
        })
        .to_string()
    }

    #[tokio::test]
    async fn publish_binance_successfully() {
        let data = generate_message_data("a", "b");
        let (tx, _rx) = channel(10);
        let result = Ok(Message::Text(data));
        assert!(
            Book::publish(tx, result).await.is_ok(),
            "failed to publish books"
        )
    }

    #[tokio::test]
    async fn publish_bitstamp_successfully() {
        let data = generate_message_data("asks", "bids");
        let (tx, _rx) = channel(10);
        let result = Ok(Message::Text(data));
        assert!(
            Book::publish(tx, result).await.is_ok(),
            "failed to publish books"
        )
    }
}
