pub mod basket;
pub mod basket_state;
pub mod ordering;

use crate::models::*;
use leptos::server;
use crate::error::AppError;
#[cfg(feature = "ssr")]
use http::StatusCode;

#[cfg(feature = "ssr")]
use std::sync::Arc;
#[cfg(feature = "ssr")]
use settings::Settings;
#[cfg(feature = "ssr")]
use leptos::prelude::expect_context;

#[cfg_attr(not(feature = "ssr"), allow(dead_code))]
fn to_server_err(e: impl std::fmt::Display) -> AppError {
    AppError(e.to_string())
}

#[cfg(feature = "ssr")]
fn catalog_base() -> String {
    expect_context::<Arc<Settings>>()
        .services
        .catalog_api
        .http
        .clone()
}

#[cfg(feature = "ssr")]
fn catalog_api_version() -> String {
    expect_context::<Arc<Settings>>()
        .services
        .catalog_api
        .version
        .clone()
        .unwrap()
}

#[server]
pub async fn get_catalog_items(
    page_index: i32,
    page_size: i32,
    brand: Option<i32>,
    type_id: Option<i32>,
) -> Result<CatalogResult, AppError> {
    let base = catalog_base();
    let version = catalog_api_version();
    let mut url = format!(
        "{}/api/catalog/items?pageIndex={}&pageSize={}&api-version={}",
        base, page_index, page_size, version,
    );
    if let Some(b) = brand {
        url.push_str(&format!("&brand={}", b));
    }
    if let Some(t) = type_id {
        url.push_str(&format!("&type={}", t));
    }
    let client = reqwest::Client::new();
    let resp = client
        .get(&url)
        .send()
        .await
        .map_err(to_server_err)?;
    let result: CatalogResult = resp.json().await.map_err(to_server_err)?;
    Ok(result)
}

#[server]
pub async fn get_catalog_brands() -> Result<Vec<CatalogBrand>, AppError> {
    let base = catalog_base();
    let version = catalog_api_version();
    let url = format!("{}/api/catalog/catalogBrands?api-version={}", base, version);
    let client = reqwest::Client::new();
    let resp = client
        .get(&url)
        .send()
        .await
        .map_err(to_server_err)?;
    let result: Vec<CatalogBrand> = resp.json().await.map_err(to_server_err)?;
    Ok(result)
}

#[server]
pub async fn get_catalog_types() -> Result<Vec<CatalogItemType>, AppError> {
    let base = catalog_base();
    let version = catalog_api_version();
    let url = format!("{}/api/catalog/catalogTypes?api-version={}", base, version);
    let client = reqwest::Client::new();
    let resp = client
        .get(&url)
        .send()
        .await
        .map_err(to_server_err)?;
    let result: Vec<CatalogItemType> = resp.json().await.map_err(to_server_err)?;
    Ok(result)
}

#[server]
pub async fn get_catalog_item(id: i32) -> Result<Option<CatalogItem>, AppError> {
    let base = catalog_base();
    let version = catalog_api_version();
    let url = format!("{}/api/catalog/items/{}?api-version={}", base, id, version);
    let client = reqwest::Client::new();
    let resp = client.get(&url).send().await.map_err(to_server_err)?;
    if resp.status() == StatusCode::NOT_FOUND {
        return Ok(None);
    }
    let result: CatalogItem = resp.json().await.map_err(to_server_err)?;
    Ok(Some(result))
}
