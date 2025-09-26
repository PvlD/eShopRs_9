use rust_decimal::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CatalogItemType {
    pub id: usize,

    #[serde(rename(serialize = "type", deserialize = "type"))]
    pub type_name: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CatalogBrand {
    pub id: usize,

    pub brand: String,
}

impl Default for CatalogItem {
    fn default() -> Self {
        Self::new()
    }
}

impl CatalogItem {
    pub fn new() -> Self {
        Self {
            id: Default::default(),
            name: String::new(),
            description: String::new(),
            price: Decimal::zero(),
            picture_url: None,
            catalog_type_id: Default::default(),
            catalog_type: None,
            catalog_brand_id: Default::default(),
            catalog_brand: None,
        }
    }
}
impl Default for CatalogBrand {
    fn default() -> Self {
        Self::new()
    }
}

impl CatalogBrand {
    pub fn new() -> Self {
        CatalogBrand { id: 0, brand: String::new() }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CatalogItem {
    pub id: i32,

    pub name: String,

    pub description: String,

    pub price: Decimal,

    pub picture_url: Option<String>,

    pub catalog_type_id: usize,

    pub catalog_type: Option<CatalogItemType>,

    pub catalog_brand_id: usize,

    pub catalog_brand: Option<CatalogBrand>,
}
