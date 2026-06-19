use async_trait::async_trait;

use crate::error::EventBusError;
use crate::event::IntegrationEvent;

#[async_trait]
pub trait EventBus: Send + Sync {
    async fn publish<T: IntegrationEvent>(&self, event: &T) -> Result<(), EventBusError>;
}

#[async_trait]
pub trait EventHandler<T: IntegrationEvent>: Send + Sync {
    async fn handle(&self, event: T);
}
