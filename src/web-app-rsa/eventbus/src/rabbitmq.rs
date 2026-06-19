use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use amqprs::{
    channel::{
        BasicPublishArguments, Channel, ExchangeDeclareArguments,
    },
    connection::{Connection, OpenConnectionArguments},
    consumer::AsyncConsumer,
    BasicProperties, Deliver,
};
use async_trait::async_trait;
use backoff::ExponentialBackoff;
use backoff::future::retry as backoff_retry;
use tokio::sync::RwLock;
use tokio::task::JoinHandle;
use tracing::{error, info, warn};

use crate::error::EventBusError;
use crate::event::IntegrationEvent;
use crate::traits::{EventBus, EventHandler};

const EXCHANGE_NAME: &str = "eshop_event_bus";

pub(crate) type HandlerFn =
    Arc<dyn Fn(String) -> Pin<Box<dyn Future<Output = ()> + Send>> + Send + Sync>;

#[derive(Debug, Clone)]
pub struct EventBusOptions {
    pub url: String,
    pub subscription_client_name: String,
    pub retry_count: u32,
}

pub struct RabbitMQEventBus {
    connection: Arc<RwLock<Connection>>,
    consumer_channel: Arc<Channel>,
    registry: Arc<HashMap<String, Vec<HandlerFn>>>,
    options: EventBusOptions,
}

impl RabbitMQEventBus {
    pub(crate) fn new(
        connection: Connection,
        consumer_channel: Channel,
        registry: HashMap<String, Vec<HandlerFn>>,
        options: EventBusOptions,
    ) -> Self {
        Self {
            connection: Arc::new(RwLock::new(connection)),
            consumer_channel: Arc::new(consumer_channel),
            registry: Arc::new(registry),
            options,
        }
    }

    pub(crate) async fn start_consumer(
        self: &Arc<Self>,
    ) -> Result<JoinHandle<()>, EventBusError> {
        let consumer = AmqpConsumer {
            bus: self.clone(),
        };

        let args = amqprs::channel::BasicConsumeArguments::new(
            &self.options.subscription_client_name,
            "",
        )
        .auto_ack(false)
        .finish();

        self.consumer_channel
            .basic_consume(consumer, args)
            .await
            .map_err(|e| EventBusError::Channel(e.to_string()))?;

        info!(
            "Event bus consumer started on queue '{}'",
            self.options.subscription_client_name
        );

        let handle = tokio::spawn({
            let bus = self.clone();
            async move {
                bus.consumer_keepalive().await;
            }
        });

        Ok(handle)
    }

    async fn consumer_keepalive(&self) {
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(3600)).await;
        }
    }

    pub(crate) async fn process_message(&self, routing_key: &str, body: &[u8]) {
        let json = match String::from_utf8(body.to_vec()) {
            Ok(s) => s,
            Err(e) => {
                error!("Invalid UTF-8 in message body: {}", e);
                return;
            }
        };

        let handlers = match self.registry.get(routing_key) {
            Some(h) => h,
            None => {
                warn!("No handlers registered for routing key '{}'", routing_key);
                return;
            }
        };

        for handler in handlers {
            handler(json.clone()).await;
        }
    }

    async fn publish_raw(&self, routing_key: &str, body: Vec<u8>) -> Result<(), EventBusError> {
        let channel = self
            .connection
            .read()
            .await
            .open_channel(None)
            .await
            .map_err(|e| EventBusError::Channel(e.to_string()))?;

        channel
            .exchange_declare(
                ExchangeDeclareArguments::of_type(EXCHANGE_NAME, amqprs::channel::ExchangeType::Direct)
                    .durable(false)
                    .finish(),
            )
            .await
            .map_err(|e| EventBusError::Channel(e.to_string()))?;

        let props = BasicProperties::default()
            .with_delivery_mode(2)
            .finish();

        let args = BasicPublishArguments::new(EXCHANGE_NAME, routing_key);

        channel
            .basic_publish(props, body, args)
            .await
            .map_err(|e| EventBusError::Channel(e.to_string()))?;

        channel
            .close()
            .await
            .map_err(|e| EventBusError::Channel(e.to_string()))?;

        Ok(())
    }
}

#[async_trait]
impl EventBus for RabbitMQEventBus {
    async fn publish<T: IntegrationEvent>(&self, event: &T) -> Result<(), EventBusError> {
        let routing_key = T::event_name();
        let body = serde_json::to_vec(event)?;

        let mut last_err = None;
        for attempt in 0..=self.options.retry_count {
            match self.publish_raw(routing_key, body.clone()).await {
                Ok(()) => return Ok(()),
                Err(e) => {
                    last_err = Some(e);
                    if attempt < self.options.retry_count {
                        let delay_ms = 100u64 * 2u64.pow(attempt);
                        tokio::time::sleep(tokio::time::Duration::from_millis(delay_ms)).await;
                    }
                }
            }
        }

        Err(last_err.unwrap_or(EventBusError::Connection(
            "publish failed".into(),
        )))
    }
}

struct AmqpConsumer {
    bus: Arc<RabbitMQEventBus>,
}

#[async_trait]
impl AsyncConsumer for AmqpConsumer {
    async fn consume(
        &mut self,
        channel: &Channel,
        deliver: Deliver,
        _basic_properties: BasicProperties,
        content: Vec<u8>,
    ) {
        let routing_key = deliver.routing_key();
        self.bus.process_message(&routing_key, &content).await;

        let ack_args = amqprs::channel::BasicAckArguments::new(deliver.delivery_tag(), false);
        if let Err(e) = channel.basic_ack(ack_args).await {
            error!("Failed to ack message: {}", e);
        }
    }
}

fn parse_amqp_url(url: &str) -> Result<(&str, u16, &str, &str), EventBusError> {
    let without_scheme = url
        .strip_prefix("amqp://")
        .ok_or_else(|| EventBusError::Configuration("URL must start with amqp://".into()))?;

    let (userinfo, rest) = without_scheme
        .split_once('@')
        .unwrap_or(("guest", without_scheme));

    let password = userinfo.split(':').nth(1).unwrap_or("guest");
    let user = userinfo.split(':').next().unwrap_or("guest");

    let host_and_port = rest.split('/').next().unwrap_or(rest);

    let (host, port_str) = host_and_port
        .split_once(':')
        .unwrap_or((host_and_port, "5672"));

    let port = port_str
        .parse::<u16>()
        .map_err(|_| EventBusError::Configuration(format!("Invalid port '{}'", port_str)))?;

    Ok((host, port, user, password))
}

pub(crate) async fn connect_with_retry(
    url: &str,
    _retry_count: u32,
) -> Result<Connection, EventBusError> {
    let url = url.to_string();

    let op = move || {
        let url = url.clone();
        async move {
            let parsed = parse_amqp_url(&url).map_err(backoff::Error::Permanent)?;
            let args = OpenConnectionArguments::new(parsed.0, parsed.1, parsed.2, parsed.3);
            Connection::open(&args).await.map_err(|e| {
                backoff::Error::Transient {
                    err: EventBusError::Connection(e.to_string()),
                    retry_after: None,
                }
            })
        }
    };

    let backoff = ExponentialBackoff {
        max_elapsed_time: None,
        ..Default::default()
    };

    backoff_retry(backoff, op).await
}

pub(crate) fn make_handler_fn<T, H>(handler: H) -> HandlerFn
where
    T: IntegrationEvent,
    H: EventHandler<T> + Clone + 'static,
{
    Arc::new(move |json: String| {
        let handler = handler.clone();
        Box::pin(async move {
            match serde_json::from_str::<T>(&json) {
                Ok(event) => handler.handle(event).await,
                Err(e) => error!(
                    "Failed to deserialize event '{}': {}",
                    T::event_name(),
                    e
                ),
            }
        }) as Pin<Box<dyn Future<Output = ()> + Send>>
    })
}
