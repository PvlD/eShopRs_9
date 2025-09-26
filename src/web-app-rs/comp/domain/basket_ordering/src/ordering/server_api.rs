use anyhow::Result;
use leptos::server;
use uuid::Uuid;

use super::types::Order;

#[cfg(feature = "ssr")]
use super::service::OrderingServiceContext;

#[cfg(feature = "ssr")]
fn use_ordering_service_context() -> Result<OrderingServiceContext, crate::AppError> {
    use leptos::prelude::{ServerFnErrorErr, use_context};
    use_context::<OrderingServiceContext>().ok_or(crate::AppError::ServerFnError(ServerFnErrorErr::ServerError("OrderingServiceContext not in context".to_string())))
}

#[server]
#[middleware(auth::RequireAuth)]

pub async fn get_orders() -> Result<Vec<Order>, crate::AppError> {
    let ctx = use_ordering_service_context()?;
    ctx.service.get_orders().await
}

use crate::basket_state::types::CreateOrderRequest;
#[server]
#[middleware(auth::RequireAuth)]
pub async fn create_order(request: CreateOrderRequest, request_id: Uuid) -> Result<(), crate::AppError> {
    let ctx = use_ordering_service_context()?;
    ctx.service.create_order(request, request_id).await
}
