use tokio_tungstenite::tungstenite;

/// Possible order book errors.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    WsError(#[from] tungstenite::Error),

    #[error(transparent)]
    ParseError(#[from] serde_json::Error),

    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}
