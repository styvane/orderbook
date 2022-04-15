//! Settings module

use secrecy::Secret;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Configuration {
    pub exchanges: Vec<ExchangeConfig>,
}

#[derive(Debug, Deserialize)]
pub struct ExchangeConfig {
    pub exchange: String,
    pub channel: String,
    pub url: String,
    pub credential: Option<Credential>,
}

#[derive(Debug, Deserialize)]
pub struct Credential {
    pub user_id: Secret<String>,
    pub token: Secret<String>,
}
