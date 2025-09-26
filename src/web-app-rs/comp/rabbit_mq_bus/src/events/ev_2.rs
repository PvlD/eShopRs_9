use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
/*
use crate::content_processor::IDispatcheable;
use crate::content_processor::{IContent, IFromContent, IKeyed};
use crate::Key;

use crate::content_processor::Dispatcher;
*/
use ebus::Content;
use ebus::Dispatcher;
use ebus::Dispatcherable;
use ebus::FromContent;
use ebus::Key;
use ebus::Keyed;

#[derive(Debug, PartialEq, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Ev2 {
    pub data: String,
    pub buyer_identity_guid: String,
}

impl Keyed for Ev2 {
    fn key() -> Key {
        "ev2"
    }
}

impl FromContent for Ev2 {
    fn from_content(data: Vec<u8>) -> Result<Self, ebus::lib_err::AppError> {
        let event = serde_json::from_slice(&data)?;
        Ok(event)
    }
}

impl Content for Ev2 {
    fn content(&self) -> Result<(Key, Vec<u8>), ebus::lib_err::AppError> {
        let json = serde_json::to_vec(&self)?;
        Ok((Ev2::key(), json))
    }
}

impl Dispatcherable<Ev2> for Ev2 {
    fn dispatcher() -> Arc<RwLock<Dispatcher<Ev2>>> {
        get_dispatcher().clone()
    }
}
use std::sync::OnceLock;
static DISPATCHER: OnceLock<Arc<RwLock<Dispatcher<Ev2>>>> = OnceLock::new();

fn get_dispatcher() -> &'static Arc<RwLock<Dispatcher<Ev2>>> {
    DISPATCHER.get_or_init(|| Arc::new(RwLock::new(Dispatcher::new())))
}
