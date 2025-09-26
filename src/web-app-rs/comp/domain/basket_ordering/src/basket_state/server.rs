use async_trait::async_trait;
use tonic::Extensions;

use crate::basket::types::BasketQuantity;
use crate::catalog::service::CatalogServiceContext;
use crate::catalog::types::CatalogItem;

use crate::basket_state::service::{BasketCheckoutInfo, BasketStateService, BasketStateServiceContext};
use leptos_axum::extract;

use crate::basket_state::types::BasketItem;
use crate::ordering::service::OrderingServiceContext;

use anyhow::Result;
use uuid::Uuid;

use crate::basket::service::BasketServiceContext;

use std::{collections::HashMap, sync::Arc};

struct BasketStateServiceApi {
    basket_service: BasketServiceContext,
    catalog_service: CatalogServiceContext,
    ordering_service: OrderingServiceContext,
}

impl BasketStateServiceApi {
    pub fn new(basket_service: BasketServiceContext, catalog_service: CatalogServiceContext, ordering_service: OrderingServiceContext) -> Self {
        BasketStateServiceApi {
            basket_service,
            catalog_service,
            ordering_service,
        }
    }

    async fn fetch_core_async(&self) -> Result<Vec<BasketItem>, crate::AppError> {
        use rust_decimal::Decimal;
        let quantities = self.basket_service.service.get_basket().await?;
        if quantities.is_empty() {
            return Ok(vec![]);
        }

        // Get details for the items in the basket
        let mut basket_items = Vec::<BasketItem>::new();
        let product_id: Vec<i32> = quantities.iter().map(|item| item.product_id).collect::<Vec<_>>();
        let catalog_items_v = self.catalog_service.service.get_catalog_items_by_ids(product_id).await.map_err(|e| crate::AppError::Other(e.to_string()))?;

        let catalog_items: HashMap<i32, &CatalogItem> = catalog_items_v.iter().map(|x| (x.id, x)).collect();

        for item in quantities {
            let catalog_item = catalog_items.get(&item.product_id).ok_or_else(|| crate::AppError::Other(format!("Catalog item not found {}", item.product_id)))?;
            let order_item = BasketItem {
                id: Uuid::new_v4().to_string(),
                product_id: catalog_item.id,
                product_name: catalog_item.name.clone(),
                unit_price: catalog_item.price,
                old_unit_price: Decimal::default(),
                quantity: item.quantity,
            };
            basket_items.push(order_item);
        }

        Ok(basket_items)
    }

    async fn fetch_basket_items(&self) -> Result<Vec<BasketItem>, crate::AppError> {
        self.fetch_core_async().await
    }

    async fn delete_basket(&self) -> Result<(), crate::AppError> {
        self.basket_service.service.delete_basket().await
    }
}

#[async_trait]
impl BasketStateService for BasketStateServiceApi {
    async fn get_basket_items(&self) -> Result<Vec<BasketItem>, crate::AppError> {
        if auth::server::is_authenticated().await? { self.fetch_basket_items().await } else { Ok(vec![]) }
    }

    async fn add_basket_item(&self, item: CatalogItem) -> Result<(), crate::AppError> {
        let mut basket_items = self
            .fetch_basket_items()
            .await?
            .iter()
            .map(|x| BasketQuantity {
                product_id: x.product_id,
                quantity: x.quantity,
            })
            .collect::<Vec<_>>();
        if let Some(basket_item) = basket_items.iter_mut().find(|x| x.product_id == item.id) {
            basket_item.quantity += 1;
        } else {
            basket_items.push(BasketQuantity { product_id: item.id, quantity: 1 });
        }
        self.basket_service.service.update_basket(basket_items).await?;
        Ok(())
    }

    async fn set_quantity(&self, product_id: i32, quantity: i32) -> Result<bool, crate::AppError> {
        let mut basket_items = self
            .fetch_basket_items()
            .await?
            .iter()
            .map(|x| BasketQuantity {
                product_id: x.product_id,
                quantity: x.quantity,
            })
            .collect::<Vec<_>>();

        let changed = if let Some(basket_item) = basket_items.iter_mut().find(|x| x.product_id == product_id) {
            if quantity > 0 {
                basket_item.quantity = quantity;
            } else {
                basket_items.retain(|x| x.product_id != product_id);
            }
            true
        } else if quantity > 0 {
            basket_items.push(BasketQuantity { product_id: product_id, quantity: quantity });
            true
        } else {
            false
        };

        if changed {
            self.basket_service.service.update_basket(basket_items).await?;
        }

        Ok(changed)
    }

    async fn checkout(&self, checkout_info: BasketCheckoutInfo) -> Result<(), crate::AppError> {
        let checkout_info = if checkout_info.request_id == Uuid::default() {
            let mut checkout_info = checkout_info;
            checkout_info.request_id = Uuid::new_v4();
            checkout_info
        } else {
            checkout_info
        };

        let extensions: Extensions = extract().await?;
        let user = auth::server::get_user_ref_from_extensions(&extensions)?;
        let user_name = user.username.clone();
        let buyer_id = user.sub.clone();

        let order_items = self.fetch_basket_items().await?;

        let order = super::types::CreateOrderRequest {
            user_id: buyer_id.clone(),
            user_name,
            city: checkout_info.city,
            street: checkout_info.street,
            state: checkout_info.state,
            country: checkout_info.country,
            zip_code: checkout_info.zip_code,
            card_number: checkout_info.card_number.unwrap_or("1111222233334444".to_string()),
            card_holder_name: checkout_info.card_holder_name.unwrap_or("TESTUSER".to_string()),
            card_security_number: checkout_info.card_security_number.unwrap_or("111".to_string()),
            card_expiration: checkout_info.card_expiration,
            card_type_id: checkout_info.card_type_id,
            buyer: buyer_id,
            items: order_items,
        };

        self.ordering_service.service.create_order(order, checkout_info.request_id).await?;
        self.delete_basket().await?;
        Ok(())
    }
}

use leptos::prelude::expect_context;
pub fn make_service_from_context() -> Result<BasketStateServiceContext> {
    Ok(BasketStateServiceContext {
        service: Arc::new(BasketStateServiceApi::new(
            expect_context::<BasketServiceContext>(),
            expect_context::<CatalogServiceContext>(),
            expect_context::<OrderingServiceContext>(),
        )),
    })
}

pub fn make_service(basket_service: BasketServiceContext, catalog_service: CatalogServiceContext, ordering_service: OrderingServiceContext) -> Result<BasketStateServiceContext> {
    Ok(BasketStateServiceContext {
        service: Arc::new(BasketStateServiceApi::new(basket_service, catalog_service, ordering_service)),
    })
}
