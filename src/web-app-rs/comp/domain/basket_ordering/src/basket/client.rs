use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use leptos::prelude::provide_context;

use crate::basket::{
    server_api,
    service::{BasketService, BasketServiceContext},
    types::BasketQuantity,
};

struct BasketServiceClient {}

#[async_trait]
impl BasketService for BasketServiceClient {
    async fn get_basket(&self) -> Result<Vec<BasketQuantity>, crate::AppError> {
        server_api::get_basket().await
    }

    async fn update_basket(&self, basket: Vec<BasketQuantity>) -> Result<(), crate::AppError> {
        server_api::update_basket(basket).await
    }

    async fn delete_basket(&self) -> Result<(), crate::AppError> {
        server_api::delete_basket().await
    }
}

pub fn provide_basket_service_context() {
    provide_context(BasketServiceContext { service: Arc::new(BasketServiceClient {}) })
}
