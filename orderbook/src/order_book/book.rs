//! Order book type.
//!
//! This module defines the data structures for maintaining an order book.

use std::cmp::Ordering;

use priority_queue::DoublePriorityQueue;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// The [`OrderBook`] type. the [See module level documentation](self).
#[derive(Debug)]
pub struct OrderBook {
    cap: usize,
    pub(super) kind: BookKind,
    pub(super) books: DoublePriorityQueue<Exchange, Book>,
}

/// The [`Book`] type represents a book in an [order book](OrderBook).
#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct Book {
    price: Decimal,
    amount: Decimal,
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

#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub enum Exchange {
    Binance,
    Bitstamp,
}

/// The [`BookKind`] type is the different kind of books in an order book.
#[derive(Debug, Serialize, Deserialize)]
pub enum BookKind {
    Ask,
    Bid,
}

impl Book {
    /// Creates new book with the specified price and amount.
    pub fn new(price: Decimal, amount: Decimal) -> Self {
        Book { price, amount }
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
    /// let mut order_book = OrderBook::with_capacity(BookKind::Ask, 1);
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
    /// let mut order_book = OrderBook::with_capacity(BookKind::Bid, 1);
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
}
