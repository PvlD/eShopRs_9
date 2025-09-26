use amqprs::BasicProperties;
use amqprs::callbacks::{DefaultChannelCallback, DefaultConnectionCallback};
use amqprs::channel::{BasicConsumeArguments, BasicPublishArguments, Channel, ExchangeDeclareArguments, QueueBindArguments, QueueDeclareArguments};
use amqprs::connection::{Connection, OpenConnectionArguments};
use async_trait::async_trait;

use crate::AMQOConfig;
use crate::lib_err::AppError;
use ebus::{Content, Keyed, KeyedContainer};

#[async_trait]
pub trait EventBus<T>: Send + Sync + 'static
where
    T: Content + Keyed + Send + 'static,
    Self: Send + Sync + 'static + Sized,
{
    async fn publish(&self, event: T) -> Result<(), AppError>;
}

#[async_trait]
pub trait EventBusFactory<TC>: Send + Sync + 'static
where
    TC: amqprs::consumer::AsyncConsumer + Send + KeyedContainer + 'static,
    Self: Send + Sync + 'static + Sized,
{
    async fn new_from_config(consumer: TC, config: crate::AMQOConfig, consumer_tag: &str) -> Result<Self, amqprs::error::Error>;
}

#[async_trait]
pub trait EventBusFactoryPublisher: Send + Sync + 'static {
    async fn new_from_config_publisher(keys: Vec<&str>, config: crate::AMQOConfig) -> Result<Self, amqprs::error::Error>
    where
        Self: Sized;
}

pub struct MqEventBus {
    exchange_name: String,
    #[allow(dead_code)]
    queue_name: String,
    connection: Connection,
    consumer_channel: Channel,
}

impl MqEventBus {
    async fn publish_internal(&self, routing_key: &str, content: Vec<u8>) -> Result<(), amqprs::error::Error> {
        let channel = self.connection.open_channel(None).await?;
        channel.register_callback(DefaultChannelCallback).await?;

        let basic_properties = BasicProperties::default().with_persistence(true).finish();

        let args = BasicPublishArguments::new(self.exchange_name.as_str(), routing_key).mandatory(true).finish();
        channel.basic_publish(basic_properties, content, args).await?;
        channel.close().await?;
        Ok(())
    }

    pub async fn stop(self) -> Result<(), amqprs::error::Error> {
        self.consumer_channel.close().await?;
        self.connection.close().await?;
        Ok(())
    }
}

#[async_trait]
impl<TC> EventBusFactory<TC> for MqEventBus
where
    TC: amqprs::consumer::AsyncConsumer + Send + KeyedContainer + 'static,
{
    async fn new_from_config(consumer: TC, config: crate::AMQOConfig, consumer_tag: &str) -> Result<Self, amqprs::error::Error> {
        let keys = consumer.keys();
        let queue_name = config.queue_name.clone();
        let epub = MqEventBus::new_from_config_publisher(keys, config).await?;

        let args = BasicConsumeArguments::new(&queue_name, consumer_tag).manual_ack(true).finish();
        epub.consumer_channel.basic_consume(consumer, args).await.unwrap();

        Ok(epub)
    }
}

impl TryFrom<&AMQOConfig> for OpenConnectionArguments {
    type Error = amqprs::error::Error;
    fn try_from(config: &AMQOConfig) -> Result<Self, Self::Error> {
        match &config.connection {
            crate::Connection::Data { host, port, username, password } => Ok(OpenConnectionArguments::new(host.as_str(), *port, username.as_str(), password.as_str())),
            crate::Connection::String(connection_string) => OpenConnectionArguments::try_from(connection_string.as_str()),
        }
    }
}

#[async_trait]
impl EventBusFactoryPublisher for MqEventBus {
    async fn new_from_config_publisher(keys: Vec<&str>, config: crate::AMQOConfig) -> Result<Self, amqprs::error::Error> {
        let connection = Connection::open(&OpenConnectionArguments::try_from(&config)?).await?;
        connection.register_callback(DefaultConnectionCallback).await.unwrap();
        let consumer_channel = connection.open_channel(None).await?;

        consumer_channel.register_callback(DefaultChannelCallback).await?;
        let args = ExchangeDeclareArguments::new(config.exchange_name.as_str(), "direct");
        let _ = consumer_channel.exchange_declare(args).await?;

        let args = QueueDeclareArguments::new(config.queue_name.as_str()).durable(true).exclusive(false).auto_delete(false).no_wait(false).finish();
        let _r = consumer_channel.queue_declare(args).await?;

        for key in keys {
            consumer_channel.queue_bind(QueueBindArguments::new(&config.queue_name, &config.exchange_name, key)).await?;
        }
        Ok(Self {
            exchange_name: config.exchange_name,
            queue_name: config.queue_name,
            connection,
            consumer_channel,
        })
    }
}

#[async_trait]
impl<T> EventBus<T> for MqEventBus
where
    T: Content + Keyed + Send + 'static,
{
    async fn publish(&self, event: T) -> Result<(), AppError> {
        let (routing_key, content) = event.content()?;
        self.publish_internal(routing_key, content).await?;
        Ok(())
    }
}
