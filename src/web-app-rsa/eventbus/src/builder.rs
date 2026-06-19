use std::collections::HashMap;
use std::sync::Arc;

use amqprs::channel::{
    ExchangeDeclareArguments, ExchangeType, QueueBindArguments, QueueDeclareArguments,
};
use tracing::info;

use crate::error::EventBusError;
use crate::event::IntegrationEvent;
use crate::rabbitmq::{
    connect_with_retry, make_handler_fn, EventBusOptions, HandlerFn, RabbitMQEventBus,
};
use crate::traits::EventHandler;

pub struct EventBusBuilder {
    options: EventBusOptions,
    registry: HashMap<String, Vec<HandlerFn>>,
}

impl EventBusBuilder {
    pub fn new() -> Self {
        Self {
            options: EventBusOptions {
                url: String::new(),
                subscription_client_name: String::from("WebApp"),
                retry_count: 10,
            },
            registry: HashMap::new(),
        }
    }

    pub fn url(mut self, url: &str) -> Self {
        self.options.url = url.to_string();
        self
    }

    pub fn subscription_client_name(mut self, name: &str) -> Self {
        self.options.subscription_client_name = name.to_string();
        self
    }

    pub fn retry_count(mut self, count: u32) -> Self {
        self.options.retry_count = count;
        self
    }

    pub fn add_subscription<T, H>(mut self, handler: H) -> Self
    where
        T: IntegrationEvent,
        H: EventHandler<T> + Clone + 'static,
    {
        let routing_key = T::event_name();
        let handler_fn = make_handler_fn::<T, H>(handler);
        self.registry
            .entry(routing_key.to_string())
            .or_default()
            .push(handler_fn);
        info!("Registered handler for event '{}'", routing_key);
        self
    }

    pub async fn build(
        self,
    ) -> Result<(Arc<RabbitMQEventBus>, tokio::task::JoinHandle<()>), EventBusError> {
        let connection = connect_with_retry(&self.options.url, self.options.retry_count).await?;

        let consumer_channel = connection
            .open_channel(None)
            .await
            .map_err(|e| EventBusError::Channel(e.to_string()))?;

        consumer_channel
            .exchange_declare(
                ExchangeDeclareArguments::of_type("eshop_event_bus", ExchangeType::Direct)
                    .durable(false)
                    .finish(),
            )
            .await
            .map_err(|e| EventBusError::Channel(e.to_string()))?;

        consumer_channel
            .queue_declare(
                QueueDeclareArguments::new(&self.options.subscription_client_name)
                    .durable(true)
                    .finish(),
            )
            .await
            .map_err(|e| EventBusError::Channel(e.to_string()))?;

        for routing_key in self.registry.keys() {
            consumer_channel
                .queue_bind(QueueBindArguments::new(
                    &self.options.subscription_client_name,
                    "eshop_event_bus",
                    routing_key,
                ))
                .await
                .map_err(|e| EventBusError::Channel(e.to_string()))?;
            info!(
                "Bound routing key '{}' to queue '{}'",
                routing_key, self.options.subscription_client_name
            );
        }

        let bus = Arc::new(RabbitMQEventBus::new(
            connection,
            consumer_channel,
            self.registry,
            self.options,
        ));

        let consumer_handle = bus.start_consumer().await?;

        Ok((bus, consumer_handle))
    }
}

impl Default for EventBusBuilder {
    fn default() -> Self {
        Self::new()
    }
}
