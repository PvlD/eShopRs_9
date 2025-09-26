mod catalog;
pub(crate) use catalog::CatalogPage;

mod catalog_list_item;

mod catalog_search;
pub(crate) use catalog_search::CatalogSearch;
use leptos::prelude::ServerFnError;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum CatalogPageError {
    #[error("Invalid Page Index")]
    InvalidPageIndex,
    #[error("Invalid Brand Id")]
    InvalidBrandId,
    #[error("Invalid Item Type Id")]
    InvalidItemTypeId,
    #[error("ServerFnError {0}")]
    ServerFnError(ServerFnError),
}

impl From<CatalogPageError> for crate::AppError {
    fn from(value: CatalogPageError) -> Self {
        crate::AppError::Other(value.to_string())
    }
}
