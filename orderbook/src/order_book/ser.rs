use std::collections::HashMap;

use serde::ser::{Serialize, SerializeStruct, Serializer};

use super::OrderBook;

impl Serialize for OrderBook {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("OrderBook", 2)?;
        state.serialize_field("kind", &self.kind)?;
        let books = self
            .books
            .clone()
            .into_sorted_iter()
            .rev()
            .collect::<HashMap<_, _>>();
        state.serialize_field("books", &books)?;
        state.end()
    }
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;
    use rust_decimal_macros::dec;

    #[test]
    fn serde_can_serialize_orderbook() {
        let mut order_book = OrderBook::with_capacity(BookKind::Ask, 2);
        let book = Book::new(dec!(2.1), dec!(0.4));
        order_book.push(Exchange::Bitstamp, book);
        insta::assert_json_snapshot!(&order_book);
    }
}
