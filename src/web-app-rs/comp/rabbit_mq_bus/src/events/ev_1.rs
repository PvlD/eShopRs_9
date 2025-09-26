use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

use ebus::Content;
use ebus::Dispatcher;
use ebus::Dispatcherable;
use ebus::FromContent;
use ebus::Key;
use ebus::Keyed;

#[derive(Debug, PartialEq, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Ev1 {
    pub data: String,
    pub buyer_identity_guid: String,
}

impl Keyed for Ev1 {
    fn key() -> Key {
        "ev1"
    }
}

impl FromContent for Ev1 {
    fn from_content(data: Vec<u8>) -> Result<Self, ebus::lib_err::AppError> {
        let event = serde_json::from_slice(&data)?;
        Ok(event)
    }
}

impl Content for Ev1 {
    fn content(&self) -> Result<(Key, Vec<u8>), ebus::lib_err::AppError> {
        let json = serde_json::to_vec(&self)?;
        Ok((Ev1::key(), json))
    }
}

impl Dispatcherable<Ev1> for Ev1 {
    fn dispatcher() -> Arc<RwLock<Dispatcher<Ev1>>> {
        get_dispatcher().clone()
    }
}

use std::sync::OnceLock;
static DISPATCHER: OnceLock<Arc<RwLock<Dispatcher<Ev1>>>> = OnceLock::new();

fn get_dispatcher() -> &'static Arc<RwLock<Dispatcher<Ev1>>> {
    DISPATCHER.get_or_init(|| Arc::new(RwLock::new(Dispatcher::new())))
}
