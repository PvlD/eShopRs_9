use leptos::{
    prelude::{FromServerFnError, ServerFnErrorErr},
    server_fn::codec::JsonEncoding,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug, Serialize, Deserialize, Clone, Eq, PartialEq)]
pub enum AppError {
    #[error("ServerFnError {0}")]
    ServerFnError(ServerFnErrorErr),
    #[error("Unauthorized")]
    Unauthorized,
    #[error("RabbitMqBusError {0}")]
    RabbitMqBusError(String),
    #[error("Other {0}")]
    Other(String),
}

impl From<String> for AppError {
    fn from(value: String) -> Self {
        AppError::Other(value)
    }
}

impl FromServerFnError for AppError {
    type Encoder = JsonEncoding;
    fn from_server_fn_error(value: ServerFnErrorErr) -> Self {
        AppError::ServerFnError(value)
    }
}

impl From<ServerFnErrorErr> for AppError {
    fn from(value: ServerFnErrorErr) -> Self {
        AppError::from_server_fn_error(value)
    }
}

impl From<reqwest::Error> for AppError {
    fn from(value: reqwest::Error) -> Self {
        AppError::Other(value.to_string())
    }
}

#[cfg(not(feature = "hydrate"))]
impl From<rabbit_mq_bus::AppError> for AppError {
    fn from(value: rabbit_mq_bus::AppError) -> Self {
        AppError::RabbitMqBusError(value.to_string())
    }
}

impl AppError {
    pub fn internal_server_error() -> AppError {
        AppError::ServerFnError(ServerFnErrorErr::ServerError("internal server error".to_string()))
    }
}
