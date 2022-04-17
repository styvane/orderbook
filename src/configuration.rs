//! Configuration types.
//!
//! This module defines various configurations data structures.

use anyhow::Context;
use config::{Config, File};
use secrecy::Secret;
use serde::Deserialize;
use std::convert::TryFrom;
use std::env;
use std::path::Path;

use crate::prelude::Error;

/// Configuration type.
#[derive(Debug, Deserialize)]
pub struct Configuration {
    pub result_size: usize,
    pub exchanges: Vec<ExchangeConfig>,
    pub server: Server,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Server {
    pub hostname: String,
    pub port: u16,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ExchangeConfig {
    pub exchange: String,
    pub channel: String,
    pub url: String,
    pub credential: Option<Credential>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Credential {
    pub user_id: Secret<String>,
    pub token: Secret<String>,
}

/// Runtime environment
enum Environment {
    Local,
    Production,
}

impl TryFrom<String> for Environment {
    type Error = Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let environment = match value.to_lowercase().as_str() {
            "local" => Self::Local,
            "production" => Self::Production,
            _ => {
                return Err(anyhow::anyhow!(format!(
                    "unsupported environment `{value}`, please use `local` or `production`"
                ))
                .into())
            }
        };

        Ok(environment)
    }
}

impl AsRef<str> for Environment {
    fn as_ref(&self) -> &str {
        match self {
            Environment::Local => "local",
            Environment::Production => "production",
        }
    }
}

impl Configuration {
    /// Creates new configuration.
    pub fn new() -> crate::prelude::Result<Self> {
        let path = env::var("CONFIG_PATH");

        let path = match path {
            Ok(ref value) => Path::new(value).to_path_buf(),
            Err(_) => {
                let current_dir =
                    env::current_dir().context("failed to determine current directory")?;
                current_dir.join("settings")
            }
        };

        let mut builder =
            Config::builder().add_source(File::from(path.join("base")).required(true));
        let environ: Environment = env::var("APP_ENVIRON")
            .unwrap_or_else(|_| "local".into())
            .try_into()
            .context("unsupported APP_ENVIRON value")?;
        builder = builder.add_source(File::from(path.join(environ.as_ref())).required(true));

        let config = builder
            .build()
            .and_then(Config::try_deserialize)
            .map_err(Error::ConfigError)?;

        Ok(config)
    }
    /// Returns server address.
    pub fn server_addr(&self) -> String {
        format!("{}:{}", self.server.hostname, self.server.port)
    }
}
