use std::sync::Arc;

use async_trait::async_trait;

use anyhow::Result;
use uuid::Uuid;

use crate::ordering::types::Order;

use crate::basket_state::types::CreateOrderRequest;

#[async_trait]
pub trait OrderingService: Send + Sync {
    async fn get_orders(&self) -> Result<Vec<Order>, crate::AppError>;

    async fn create_order(&self, request: CreateOrderRequest, request_id: Uuid) -> Result<(), crate::AppError>;
}

#[derive(Clone)]
pub struct OrderingServiceContext {
    pub service: Arc<dyn OrderingService>,
}
