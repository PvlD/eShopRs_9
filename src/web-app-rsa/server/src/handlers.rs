use std::sync::Arc;

use async_trait::async_trait;
use tracing::info;

use eventbus::{EventHandler, IntegrationEvent};

use crate::notification::OrderStatusNotificationService;

#[derive(Clone)]
pub struct OrderStatusChangedHandler;

#[async_trait]
impl<T: IntegrationEvent> EventHandler<T> for OrderStatusChangedHandler {
    async fn handle(&self, event: T) {
        info!(
            event_type = %T::event_name(),
            event_id = %event.id(),
            "Order status changed event received"
        );
    }
}

#[derive(Clone)]
pub struct OrderStatusNotificationHandler {
    pub service: Arc<OrderStatusNotificationService>,
}

impl OrderStatusNotificationHandler {
    pub fn new(service: Arc<OrderStatusNotificationService>) -> Self {
        Self { service }
    }
}

#[async_trait]
impl<T: IntegrationEvent> EventHandler<T> for OrderStatusNotificationHandler {
    async fn handle(&self, event: T) {
        let buyer_id = event.buyer_id();
        info!(buyer_id = %buyer_id, "Notifying order status change");
        self.service.notify(buyer_id).await;
    }
}
