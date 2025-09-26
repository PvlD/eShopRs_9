use crate::types::{CatalogBrand, CatalogItem, CatalogItemType};
use anyhow::Result;
use leptos::server;

use crate::service::*;

#[cfg(feature = "ssr")]
use leptos::prelude::ServerFnErrorErr;

#[cfg(feature = "ssr")]
fn use_catalog_service_context() -> Result<CatalogServiceContext, crate::AppError> {
    use leptos::context::use_context;
    use_context::<CatalogServiceContext>().ok_or(crate::AppError::ServerFnError(ServerFnErrorErr::ServerError("CatalogServiceContext not in context".to_string())))
}

#[server]
pub async fn get_catalog_items(page_index: usize, page_size: usize, brand: Option<usize>, type_id: Option<usize>) -> Result<CatalogResult, crate::AppError> {
    let ctx = use_catalog_service_context()?;
    //let ctx = expect_context::<CatalogServiceContext>();
    ctx.service
        .get_catalog_items(page_index, page_size, brand, type_id)
        .await
        .map_err(|e| crate::AppError::ServerFnError(ServerFnErrorErr::ServerError(e.to_string())))
}

#[server]
pub async fn get_brands() -> Result<Vec<CatalogBrand>, crate::AppError> {
    let ctx = use_catalog_service_context()?;
    ctx.service.get_brands().await.map_err(|e| crate::AppError::ServerFnError(ServerFnErrorErr::ServerError(e.to_string())))
}

#[server]
pub async fn get_types() -> Result<Vec<CatalogItemType>, crate::AppError> {
    let ctx = use_catalog_service_context()?;
    ctx.service.get_types().await.map_err(|e| crate::AppError::ServerFnError(ServerFnErrorErr::ServerError(e.to_string())))
}

#[server]
pub async fn get_catalog_item(item_id: usize) -> Result<Option<CatalogItem>, crate::AppError> {
    let ctx = use_catalog_service_context()?;
    ctx.service.get_catalog_item(item_id).await.map_err(|e| crate::AppError::ServerFnError(ServerFnErrorErr::ServerError(e.to_string())))
}
