use crate::lib_err::*;
use async_trait::async_trait;

pub type Key = &'static str;
pub trait Keyed {
    fn key() -> Key;
}

pub trait KeyedContainer {
    fn keys(&self) -> Vec<&'static str>;
}

pub trait Content {
    fn content(&self) -> Result<(&str, Vec<u8>), AppError>;
}

pub trait FromContent {
    fn from_content(data: Vec<u8>) -> Result<Self, crate::AppError>
    where
        Self: Sized;
}

#[async_trait]
pub trait KeyedContentProcessor {
    async fn process(&self, key: &str, content: Vec<u8>) -> Result<(), crate::AppError>;
}

#[async_trait]
pub trait EventContentProcessor: Send + Sync + 'static {
    async fn process(&self, content: Vec<u8>) -> Result<(), crate::AppError>;
}
