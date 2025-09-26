use async_trait::async_trait;
use std::sync::Arc;
use tonic::{Extensions, IntoRequest, metadata::MetadataMap, transport::Channel};

use anyhow::Result;

use crate::basket::{service::*, types::BasketQuantity};
use url::Url;

use leptos::prelude::ServerFnErrorErr;
use leptos_axum::extract;

pub mod basket_grpc {
    tonic::include_proto!("basket_api");
}

use basket_grpc::{BasketItem, DeleteBasketRequest, GetBasketRequest, UpdateBasketRequest, basket_client::BasketClient};

use url_mapper::UrlMapService;
type BasketClientGrpc = BasketClient<Channel>;

struct BasketServiceApi {
    client: BasketClientGrpc,
    #[allow(dead_code)]
    base_url: Url,
    //api_version: QueryStringApiVersion,
}

const BASKET_SERVICE_PATH: &str = "api/basket";
const KEY: &str = "http://basket-api";

impl BasketServiceApi {
    pub fn new(client: BasketClientGrpc, base_url: Url) -> Self {
        BasketServiceApi { client, base_url }
    }
}

async fn authorization_delegating(metadata: &mut MetadataMap) -> Result<(), crate::AppError> {
    let extensions: Extensions = extract().await?;
    let auth_session = extensions.get::<auth::users::AuthSession>().ok_or(crate::AppError::Unauthorized)?;
    let t = auth_session.user.as_ref().ok_or(crate::AppError::Unauthorized)?.access_token.clone();
    if metadata.append("authorization", format!("Bearer {}", t).parse().unwrap()) {
        return Err(crate::AppError::Other("Authorization is already  present".to_string()));
    }
    Ok(())
}

#[async_trait]
impl BasketService for BasketServiceApi {
    async fn get_basket(&self) -> Result<Vec<BasketQuantity>, crate::AppError> {
        let mut request = GetBasketRequest {}.into_request();
        authorization_delegating(&mut request.metadata_mut()).await?;
        let mut client = self.client.clone();
        let response = client.get_basket(request).await.map_err(|e| crate::AppError::ServerFnError(ServerFnErrorErr::ServerError(e.to_string())))?;
        Ok(response
            .into_inner()
            .items
            .into_iter()
            .map(|item| BasketQuantity {
                product_id: item.product_id,
                quantity: item.quantity,
            })
            .collect())
    }

    async fn update_basket(&self, items: Vec<BasketQuantity>) -> Result<(), crate::AppError> {
        let items = items
            .into_iter()
            .map(|item| BasketItem {
                product_id: item.product_id,
                quantity: item.quantity,
            })
            .collect();
        let mut request = UpdateBasketRequest { items }.into_request();
        authorization_delegating(&mut request.metadata_mut()).await?;
        let mut client = self.client.clone();
        let _response = client.update_basket(request).await.map_err(|e| crate::AppError::ServerFnError(ServerFnErrorErr::ServerError(e.to_string())))?;
        Ok(())
    }

    async fn delete_basket(&self) -> Result<(), crate::AppError> {
        let mut request = DeleteBasketRequest {}.into_request();
        authorization_delegating(&mut request.metadata_mut()).await?;
        let mut client = self.client.clone();
        let _response = client.delete_basket(request).await.map_err(|e| crate::AppError::ServerFnError(ServerFnErrorErr::ServerError(e.to_string())))?;
        Ok(())
    }
}

pub async fn make_service(url_map_service: UrlMapService) -> Result<BasketServiceContext> {
    let base_url = url_map_service.get_mapped_url(KEY).unwrap_or(KEY);

    print!("mapped  base_url {:?}", base_url);
    let base_url = Url::parse(base_url)?;
    let base_url = base_url.join(BASKET_SERVICE_PATH)?;

    let client = BasketClient::connect(base_url.to_string()).await?;

    Ok(BasketServiceContext {
        service: Arc::new(BasketServiceApi::new(client, base_url)),
    })
}
