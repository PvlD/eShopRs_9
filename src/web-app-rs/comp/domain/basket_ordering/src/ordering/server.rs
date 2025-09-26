use anyhow::Result;
use std::sync::Arc;

use crate::basket_state::types::CreateOrderRequest;
use crate::ordering::service::OrderingService;
use crate::ordering::types::Order;
use api_version::versioning::QueryStringApiVersion;
use async_trait::async_trait;
pub use reqwest::Client as HttpClient;
use url::Url;
use url_mapper::UrlMapService;
use uuid::Uuid;

struct OrderingServiceApi {
    http_client: HttpClient,
    base_url: Url,
    api_version: QueryStringApiVersion,
}

const ORDERING_SERVICE_BASE_URL: &str = "api/orders/";
const BASE_MAP_TO_PATH: &str = "http://ordering-api";

impl OrderingServiceApi {
    pub fn new(http_client: HttpClient, base_url: Url, api_version: QueryStringApiVersion) -> Self {
        OrderingServiceApi { http_client, base_url, api_version }
    }
}

#[async_trait]
impl OrderingService for OrderingServiceApi {
    async fn get_orders(&self) -> Result<Vec<Order>, crate::AppError> {
        let mut uri = self.base_url.clone();
        self.api_version.append_to_url(&mut uri);

        let r = self.http_client.get(uri).bearer_auth(auth::server::token_from_auth_session().await?).send().await?.error_for_status()?.json::<Vec<Order>>().await?;

        Ok(r)
    }

    async fn create_order(&self, request: CreateOrderRequest, request_id: Uuid) -> Result<(), crate::AppError> {
        let mut uri = self.base_url.clone();
        self.api_version.append_to_url(&mut uri);

        let _r = self
            .http_client
            .post(uri)
            .bearer_auth(auth::server::token_from_auth_session().await?)
            .header("x-requestid", request_id.to_string())
            .json(&request)
            .send()
            .await?
            .error_for_status()?;

        Ok(())
    }
}

pub async fn make_service(http_client: HttpClient, url_map_service: UrlMapService, api_version: QueryStringApiVersion) -> Result<super::service::OrderingServiceContext> {
    let base_url = url_map_service.get_mapped_url(BASE_MAP_TO_PATH).unwrap_or(BASE_MAP_TO_PATH);

    print!("mapped  base_url {:?}", base_url);
    let base_url = Url::parse(base_url)?;
    let base_url = base_url.join(ORDERING_SERVICE_BASE_URL)?;

    Ok(super::service::OrderingServiceContext {
        service: Arc::new(OrderingServiceApi::new(http_client, base_url, api_version)),
    })
}
