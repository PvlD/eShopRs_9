use thiserror::Error;

#[derive(Debug, Error)]
pub enum EventBusError {
    #[error("Connection error: {0}")]
    Connection(String),

    #[error("Channel error: {0}")]
    Channel(String),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Configuration error: {0}")]
    Configuration(String),
}
