use rust_decimal::Decimal;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct EventText {
    #[serde(alias = "a")]
    pub bids: Vec<(Decimal, Decimal)>,
    #[serde(alias = "b")]
    pub asks: Vec<(Decimal, Decimal)>,
}
