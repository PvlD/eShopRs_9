use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum ServiceError {
    #[error("Context Error {0}")]
    ContextError(String),
}
