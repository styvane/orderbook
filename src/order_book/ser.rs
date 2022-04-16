use serde::ser::{Serialize, SerializeStruct, Serializer};

use super::{BookKind, BookQueue};

impl Serialize for BookQueue {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("BookQueue", 2)?;
        let books = self
            .books
            .clone()
            .into_sorted_iter()
            .rev()
            .map(|(_, b)| b)
            .collect::<Vec<_>>();

        let kind = match self.kind {
            BookKind::Asks => "asks",
            BookKind::Bids => "bids",
        };
        state.serialize_field(kind, &books)?;
        state.end()
    }
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;

    #[test]
    fn serde_can_serialize_orderbook() {
        let mut order_book = BookQueue::with_capacity(BookKind::Asks, 2);
        let book = Book::new("2.1", "0.4", "bitstamp");
        order_book.push(Exchange::Bitstamp, book);
        let book = Book::new("3.1", "0.1", "binance");
        order_book.push(Exchange::Binance, book);
        insta::assert_json_snapshot!(&order_book);
    }
}
