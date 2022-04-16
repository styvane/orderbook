//! Order book type.
//!
//! This module defines the data structures for maintaining an order book.

use std::cmp::Ordering;

use anyhow::Context;
use priority_queue::DoublePriorityQueue;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;

use crate::integration::event::Event;
use crate::prelude::{Error, Exchange};

/// The [`OrderBook`] type. the [See module level documentation](self).
#[derive(Debug)]
pub struct OrderBook {
    cap: usize,
    pub(super) kind: BookKind,
    pub(super) books: DoublePriorityQueue<Exchange, Book>,
}

/// The [`Book`] type represents a book in an [order book](OrderBook).
#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
#[serde(rename_all(serialize = "snake_case"))]
pub struct Book {
    price: Decimal,
    amount: Decimal,
    #[serde(skip_deserializing)]
    pub(super) exchange: Option<Exchange>,
}

impl PartialOrd for Book {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Book {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.amount.cmp(&other.amount) {
            Ordering::Equal => self.price.cmp(&other.price),
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

impl Book {
    /// Creates new book with the specified price and amount.
    pub fn new(price: Decimal, amount: Decimal) -> Self {
        Book {
            price,
            amount,
            exchange: None,
        }
    }

    /// Creates new book with the specified price and amount and exchange.
    pub fn with_exchange(price: Decimal, amount: Decimal, exchange: &Exchange) -> Self {
        Book {
            price,
            amount,
            exchange: Some(exchange.clone()),
        }
    }

    /// Set the exchange value to the specified value.
    pub fn set_exchange(&mut self, which: Exchange) {
        self.exchange = Some(which);
    }

    /// Publishes books to a channel.
    #[tracing::instrument(name = "Publishes books to a channel", skip(sender, messages))]
    pub async fn publish(
        sender: mpsc::Sender<Self>,
        messages: Result<tungstenite::Message, tungstenite::Error>,
    ) -> Result<(), Error> {
        let messages = messages?;
        let exchange: Exchange;
        let data = match serde_json::from_str::<Event>(&messages.into_text()?)? {
            Event::Binance(event) => {
                exchange = Exchange::Binance;
                event
            }
            Event::Bitstamp { data } => {
                exchange = Exchange::Bitstamp;
                data
            }
        };

        for (price, amount) in data.bids {
            let book = Book::with_exchange(price, amount, &exchange);
            sender.send(book).await.context("failed to send book")?;
        }
        Ok(())
    }
}

impl OrderBook {
    /// Creates new order book of the specified kind.
    pub fn new(kind: BookKind) -> Self {
        OrderBook {
            cap: 0,
            kind,
            books: DoublePriorityQueue::new(),
        }
    }

    /// Creates new order book of the specified kind with the specified capacity.
    pub fn with_capacity(kind: BookKind, capacity: usize) -> Self {
        OrderBook {
            cap: capacity,
            kind,
            books: DoublePriorityQueue::with_capacity(capacity),
        }
    }

    /// Adds a book to the order book.
    ///
    /// # Example
    ///
    /// ```no_run
    ///
    /// use rust_decimal_macros::dec;
    ///
    /// use orderbook::prelude::*;
    ///
    /// let mut order_book = OrderBook::with_capacity(BookKind::Asks, 1);
    /// let book = Book::new(dec!(2.1), dec!(0.4));
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
    /// ```no_run
    ///
    /// use rust_decimal_macros::dec;
    ///
    /// use orderbook::prelude::*;
    ///
    /// let mut order_book = OrderBook::with_capacity(BookKind::Bids, 1);
    /// order_book.push(Exchange::Binance, Book::new(dec!(1.9), dec!(3.7)));
    /// order_book.push(Exchange::Bitstamp, Book::new(dec!(2.5), dec!(4.1)));
    /// let value = Some((Exchange::Binance, Book::new(dec!(1.9), dec!(3.7))));
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
            .map(|(e, mut b)| {
                b.exchange = Some(e);
                b
            })
            .collect::<Vec<_>>()
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
