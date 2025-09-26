use crate::service::{CatalogResult, CatalogService, CatalogServiceContext};
use async_trait::async_trait;
use std::sync::Arc;

use anyhow::Result;
pub use reqwest::Client as HttpClient;

use crate::types::{CatalogBrand, CatalogItem, CatalogItemType};
use reqwest::Url;

use url_mapper::UrlMapService;

use api_version::versioning::QueryStringApiVersion;

struct CatalogServiceApi {
    http_client: HttpClient,
    base_url: Url,
    api_version: QueryStringApiVersion,
}

const CATALOG_SERVICE_BASE_URL: &str = "api/catalog/";
const BASE_MAP_TO_PATH: &str = "http://catalog-api";

fn get_all_catalog_items_uri(base_uri: &Url, page_index: usize, page_size: usize, brand: Option<usize>, type_id: Option<usize>) -> Result<Url> {
    let filter_qs = match (brand, type_id) {
        (Some(brand), Some(type_id)) => format!("/type/{type_id}/brand/{brand}"),
        (None, Some(type_id)) => format!("/type/{type_id}/brand/"),
        (Some(brand), None) => format!("/type/all/brand/{brand}"),
        _ => "".to_string(),
    };
    let u = format!("items{filter_qs}?pageIndex={page_index}&pageSize={page_size}");
    Ok(base_uri.join(&u)?)
}

impl CatalogServiceApi {
    pub fn new(http_client: HttpClient, base_url: Url, api_version: QueryStringApiVersion) -> Self {
        CatalogServiceApi { http_client, base_url, api_version }
    }
}

#[async_trait]
impl CatalogService for CatalogServiceApi {
    async fn get_catalog_items(&self, page_index: usize, page_size: usize, brand: Option<usize>, type_id: Option<usize>) -> Result<CatalogResult> {
        let mut uri = get_all_catalog_items_uri(&self.base_url, page_index, page_size, brand, type_id)?;
        log::info!("uri: {}", uri);

        self.api_version.append_to_url(&mut uri);

        let r = self.http_client.get(uri).send().await?.error_for_status()?.json::<CatalogResult>().await?;

        Ok(r)
    }

    async fn get_brands(&self) -> Result<Vec<CatalogBrand>> {
        let mut uri = self.base_url.join("catalogBrands")?;

        self.api_version.append_to_url(&mut uri);

        let r = self.http_client.get(uri).send().await?.error_for_status()?.json::<Vec<CatalogBrand>>().await?;

        Ok(r)
    }

    async fn get_types(&self) -> Result<Vec<CatalogItemType>> {
        let mut uri = self.base_url.join("catalogTypes")?;
        self.api_version.append_to_url(&mut uri);

        let r = self.http_client.get(uri).send().await?.error_for_status()?.json::<Vec<CatalogItemType>>().await?;

        Ok(r)
    }

    async fn get_catalog_item(&self, item_id: usize) -> Result<Option<CatalogItem>> {
        let mut uri = self.base_url.join(format!("items/{item_id}").as_str())?;
        self.api_version.append_to_url(&mut uri);

        let r = self.http_client.get(uri).send().await?;
        if r.status() == reqwest::StatusCode::NOT_FOUND {
            Ok(None) // Return None if the item is not found
        } else {
            Ok(Some(r.error_for_status()?.json::<CatalogItem>().await?))
        }
    }

    async fn get_catalog_items_by_ids(&self, item_ids: Vec<i32>) -> Result<Vec<CatalogItem>> {
        let item_ids = item_ids.iter().map(|x| x.to_string()).collect::<Vec<_>>().join("&ids=");
        let mut uri = self.base_url.join(format!("items/by?ids={}", item_ids).as_str())?;
        self.api_version.append_to_url(&mut uri);

        let r = self.http_client.get(uri).send().await?.error_for_status()?.json::<Vec<CatalogItem>>().await?;

        Ok(r)
    }
}

pub fn make_service(http_client: HttpClient, url_map_service: UrlMapService, api_version: QueryStringApiVersion) -> Result<CatalogServiceContext> {
    let base_url = url_map_service.get_mapped_url(BASE_MAP_TO_PATH).unwrap_or(BASE_MAP_TO_PATH);

    print!("mapped  base_url {:?}", base_url);
    let base_url = Url::parse(base_url)?;
    let base_url = base_url.join(CATALOG_SERVICE_BASE_URL)?;

    Ok(CatalogServiceContext {
        service: Arc::new(CatalogServiceApi::new(http_client, base_url, api_version)),
    })
}
