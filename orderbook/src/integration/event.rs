use rust_decimal::Decimal;
use serde::Deserialize;

#[non_exhaustive]
#[derive(Debug, serde::Deserialize)]
#[serde(untagged)]
pub enum Event {
    Binance(EventData),
    Bitstamp { data: EventData },
}

#[derive(Debug, Deserialize)]
pub struct EventData {
    #[serde(alias = "a")]
    pub bids: Vec<(Decimal, Decimal)>,
    #[serde(alias = "b")]
    pub asks: Vec<(Decimal, Decimal)>,
}
