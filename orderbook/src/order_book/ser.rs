use serde::ser::{Serialize, SerializeStruct, Serializer};

use super::{BookKind, OrderBook};

impl Serialize for OrderBook {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("OrderBook", 2)?;
        let books = self
            .books
            .clone()
            .into_sorted_iter()
            .rev()
            .map(|(e, mut b)| {
                b.exchange = Some(e);
                b
            })
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
    use rust_decimal_macros::dec;

    #[test]
    fn serde_can_serialize_orderbook() {
        let mut order_book = OrderBook::with_capacity(BookKind::Asks, 2);
        let book = Book::new(dec!(2.1), dec!(0.4));
        order_book.push(Exchange::Bitstamp, book);
        let book = Book::new(dec!(3.1), dec!(0.1));
        order_book.push(Exchange::Binance, book);
        insta::assert_json_snapshot!(&order_book);
    }
}
