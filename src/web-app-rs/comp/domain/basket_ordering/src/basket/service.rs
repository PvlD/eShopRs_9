use std::sync::Arc;

use async_trait::async_trait;

use crate::basket::types::BasketQuantity;

#[async_trait]
pub trait BasketService: Send + Sync {
    async fn get_basket(&self) -> Result<Vec<BasketQuantity>, crate::AppError>;

    async fn update_basket(&self, basket: Vec<BasketQuantity>) -> Result<(), crate::AppError>;

    async fn delete_basket(&self) -> Result<(), crate::AppError>;
}
#[derive(Clone)]
pub struct BasketServiceContext {
    pub service: Arc<dyn BasketService>,
}
