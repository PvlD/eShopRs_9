use std::sync::Arc;

use async_trait::async_trait;
use uuid::Uuid;

use super::types::BasketItem;
use crate::catalog::types::CatalogItem;

pub struct BasketCheckoutInfo {
    pub street: String,
    pub city: String,
    pub state: String,
    pub country: String,
    pub zip_code: String,
    pub card_number: Option<String>,
    pub card_holder_name: Option<String>,
    pub card_security_number: Option<String>,
    pub card_expiration: chrono::DateTime<chrono::Utc>,
    pub card_type_id: i32,
    pub buyer: Option<String>,
    pub request_id: Uuid,
}

#[async_trait]
pub trait BasketStateService: Send + Sync {
    async fn get_basket_items(&self) -> Result<Vec<BasketItem>, crate::AppError>;

    async fn add_basket_item(&self, item: CatalogItem) -> Result<(), crate::AppError>;
    async fn set_quantity(&self, product_id: i32, quantity: i32) -> Result<bool, crate::AppError>;
    async fn checkout(&self, checkout_info: BasketCheckoutInfo) -> Result<(), crate::AppError>;
}
#[derive(Clone)]
pub struct BasketStateServiceContext {
    pub service: Arc<dyn BasketStateService>,
}
