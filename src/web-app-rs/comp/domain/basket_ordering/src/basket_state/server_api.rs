use crate::catalog::types::CatalogItem;

use leptos::server;

#[cfg(feature = "ssr")]
use crate::basket_state::service::BasketStateServiceContext;

use crate::basket_state::types::BasketItem;

#[cfg(feature = "ssr")]
use leptos::prelude::expect_context;

#[server(prefix = "/api_basket_state")]
#[middleware(auth::RequireAuth)]
pub async fn get_basket_items() -> Result<Vec<BasketItem>, crate::AppError> {
    let context: BasketStateServiceContext = expect_context();

    context.service.get_basket_items().await
}

#[server(prefix = "/api_basket_state")]
#[middleware(auth::RequireAuth)]
pub async fn add_basket_item(item: CatalogItem) -> Result<(), crate::AppError> {
    let context: BasketStateServiceContext = expect_context();

    context.service.add_basket_item(item).await
}

#[server(prefix = "/api_basket_state")]
#[middleware(auth::RequireAuth)]
pub async fn set_quantity(product_id: i32, quantity: i32) -> Result<bool, crate::AppError> {
    let context: BasketStateServiceContext = expect_context();

    context.service.set_quantity(product_id, quantity).await
}
