//! Telemetry types.
//!
//! This module implements the Tracer type and utilities.

use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_subscriber::fmt::MakeWriter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::{EnvFilter, Registry};

pub struct Tracer<'s> {
    name: &'s str,
    env_filter: &'s str,
}

impl<'s> Tracer<'s> {
    pub fn new(name: &'s str, env_filter: &'s str) -> Self {
        Self { name, env_filter }
    }

    /// Initializes tracer.
    pub fn init<Sink>(&self, sink: Sink)
    where
        Sink: for<'w> MakeWriter<'w> + Send + Sync + 'static,
    {
        let formatting_layer = BunyanFormattingLayer::new(self.name.into(), sink);
        let filter =
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(self.env_filter));

        let subscriber = Registry::default()
            .with(filter)
            .with(JsonStorageLayer)
            .with(formatting_layer);
        tracing::subscriber::set_global_default(subscriber)
            .expect("failed to set global log subscriber");
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use once_cell::sync::Lazy;

    static TRACER: Lazy<()> = Lazy::new(|| {
        if std::env::var("TEST_LOG").is_ok() {
            Tracer::new("orderbook", "debug").init(std::io::stdout);
        }
    });

    pub fn force_lazy() {
        Lazy::force(&TRACER);
    }
}
