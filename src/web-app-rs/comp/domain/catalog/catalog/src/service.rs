use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::types::{CatalogBrand, CatalogItem, CatalogItemType};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CatalogResult {
    //#[serde(rename(deserialize = "PageIndex"))]
    pub page_index: usize,
    //#[serde(rename(deserialize = "PageSize"))]
    pub page_size: usize,
    //#[serde(rename(deserialize = "Count"))]
    pub count: usize,
    //#[serde(rename(deserialize = "Data"))]
    pub data: Vec<CatalogItem>,
}

#[async_trait]
pub trait CatalogService: Send + Sync {
    async fn get_catalog_items(&self, page_index: usize, page_size: usize, brand: Option<usize>, type_id: Option<usize>) -> Result<CatalogResult>;

    async fn get_brands(&self) -> Result<Vec<CatalogBrand>>;

    async fn get_types(&self) -> Result<Vec<CatalogItemType>>;

    async fn get_catalog_item(&self, item_id: usize) -> Result<Option<CatalogItem>>;

    async fn get_catalog_items_by_ids(&self, item_ids: Vec<i32>) -> Result<Vec<CatalogItem>>;
}

#[derive(Clone)]
pub struct CatalogServiceContext {
    pub service: Arc<dyn CatalogService>,
}
