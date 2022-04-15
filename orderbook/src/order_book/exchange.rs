//! Exchange type.
//!
//! This module implements the exchange data structure and operation.

use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::prelude::Error;

#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all(serialize = "snake_case"))]
pub enum Exchange {
    Binance,
    Bitstamp,
}

impl FromStr for Exchange {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let exchange = match s.to_lowercase().as_str() {
            "binance" => Self::Binance,
            "bitstamp" => Self::Bitstamp,
            _ => return Err(anyhow::anyhow!(format!("unsupported exchange: {s}")).into()),
        };

        Ok(exchange)
    }
}
