use leptos::prelude::*;

use leptos_meta::Title;
use leptos_router::{
    hooks::{self, use_params},
    params::Params,
};

use serde::{Deserialize, Serialize};
use thiserror::Error;

mod item;
pub(crate) use item::ItemPage;

#[derive(Error, Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum ItemPageError {
    #[error("Invalid Item Index")]
    InvalidItemIndex,
}

impl From<ItemPageError> for crate::AppError {
    fn from(value: ItemPageError) -> Self {
        crate::AppError::Other(value.to_string())
    }
}
