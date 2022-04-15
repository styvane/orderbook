pub use super::error::Error;
pub use super::order_book::*;

pub type Result<T> = std::result::Result<T, Error>;
