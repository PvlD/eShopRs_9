use std::sync::Arc;

use amqprs::{
    BasicProperties, Deliver,
    channel::{BasicAckArguments, Channel},
};
use async_trait::async_trait;
use rabbit_mq_bus::{KeyedContainer, KeyedContentProcessor};

pub struct Consumer<T: KeyedContentProcessor + KeyedContainer + Send + Sync + 'static> {
    processor: Arc<T>,
}

impl<T: KeyedContentProcessor + KeyedContainer + Send + Sync + 'static> Consumer<T> {
    pub fn new(processor: Arc<T>) -> Self {
        Self { processor }
    }
}

impl<T: KeyedContentProcessor + KeyedContainer + Send + Sync + 'static> KeyedContainer for Consumer<T> {
    fn keys(&self) -> Vec<&'static str> {
        self.processor.keys()
    }
}

#[async_trait]
impl<T: KeyedContentProcessor + KeyedContainer + Send + Sync + 'static> amqprs::consumer::AsyncConsumer for Consumer<T> {
    async fn consume(&mut self, channel: &Channel, deliver: Deliver, _basic_properties: BasicProperties, content: Vec<u8>) {
        #[cfg(feature = "traces")]
        tracing::info!("message received {} {} {}", deliver.delivery_tag(), deliver.routing_key(), content.len());
        let r = self.processor.process(deliver.routing_key(), content).await;
        if r.is_err() {
            #[cfg(feature = "traces")]
            tracing::error!("error processing message {} {} {:?}", deliver.delivery_tag(), deliver.routing_key(), r);
            log::error!("error processing message {} {} {:?}", deliver.delivery_tag(), deliver.routing_key(), r);
        }
        let r = channel.basic_ack(BasicAckArguments::new(deliver.delivery_tag(), false)).await;
        if r.is_err() {
            #[cfg(feature = "traces")]
            tracing::error!("error acknowledging message {} {} {:?}", deliver.delivery_tag(), deliver.routing_key(), r);
            log::error!("error acknowledging message {} {} {:?}", deliver.delivery_tag(), deliver.routing_key(), r);
        }
    }
}
