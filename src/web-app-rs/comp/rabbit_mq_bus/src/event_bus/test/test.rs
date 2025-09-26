#[cfg(test)]
mod tests {
    use std::{sync::Arc, time::Duration};

    use amqprs::{
        BasicProperties, Deliver,
        channel::{BasicAckArguments, Channel},
    };
    use async_trait::async_trait;
    use testcontainers::ImageExt;
    use testcontainers::{
        ContainerAsync, GenericImage,
        core::{IntoContainerPort, WaitFor},
        runners::AsyncRunner,
    };

    use crate::{AMQOConfig, MqEventBus, trace_init};
    use crate::{EventBus, EventBusFactory, EventBusFactoryPublisher};
    use ebus::{ContentProcessor, Dispatcherable, Keyed, KeyedContainer, KeyedContentProcessor, Unsubsriber};

    struct TestConsumer<T: KeyedContentProcessor + Send + Sync + 'static> {
        processor: Arc<T>,
    }

    impl<T: KeyedContentProcessor + Send + Sync + 'static> TestConsumer<T> {
        pub fn new(processor: Arc<T>) -> Self {
            Self { processor }
        }
    }

    impl<T: KeyedContentProcessor + KeyedContainer + Send + Sync + 'static> KeyedContainer for TestConsumer<T> {
        fn keys(&self) -> Vec<&'static str> {
            self.processor.keys()
        }
    }

    #[async_trait]
    impl<T: KeyedContentProcessor + Send + Sync + 'static> amqprs::consumer::AsyncConsumer for TestConsumer<T> {
        async fn consume(&mut self, channel: &Channel, deliver: Deliver, _basic_properties: BasicProperties, content: Vec<u8>) {
            #[cfg(feature = "traces")]
            tracing::info!("consume message received {} {} {}", deliver.delivery_tag(), deliver.routing_key(), content.len());
            let r = self.processor.process(deliver.routing_key(), content).await;
            if r.is_err() {
                #[cfg(feature = "traces")]
                tracing::error!("consume error processing message {} {} {:?}", deliver.delivery_tag(), deliver.routing_key(), r);
            }
            let r = channel.basic_ack(BasicAckArguments::new(deliver.delivery_tag(), false)).await;
            if r.is_err() {
                #[cfg(feature = "traces")]
                tracing::error!("consume acknowledge error message {} {} {:?}", deliver.delivery_tag(), deliver.routing_key(), r);
            }
        }
    }

    async fn start_rabbitmq_container() -> (ContainerAsync<GenericImage>, AMQOConfig) {
        match dotenvy::dotenv() {
            Ok(path) => println!(".env read successfully from {}", path.display()),
            Err(e) => println!(
                "Could not load .env file: {e}. \nProceeding assuming variables are set in the \
                 environment."
            ),
        };

        let amqo_config = AMQOConfig::from_env();

        let username = match &amqo_config.connection {
            crate::Connection::Data { username, .. } => username.clone(),
            crate::Connection::String(_) => "guest".to_string(),
        };
        let password = match &amqo_config.connection {
            crate::Connection::Data { password, .. } => password.clone(),
            crate::Connection::String(_) => "guest".to_string(),
        };

        let container = GenericImage::new("rabbitmq", "4-management")
            .with_exposed_port(5672.tcp())
            .with_exposed_port(15672.tcp())
            .with_wait_for(WaitFor::message_on_stdout("Time to start RabbitMQ:"))
            .with_env_var("RABBITMQ_USERNAME", username.clone())
            .with_env_var("RABBITMQ_PASSWORD", password.clone())
            .with_startup_timeout(Duration::from_secs(60))
            .start()
            .await
            .unwrap();

        let amqp_host_port = container.get_host_port_ipv4(5672).await.unwrap();
        let _ui_host_port = container.get_host_port_ipv4(15672).await.unwrap();
        let host = container.get_host().await.unwrap();

        let mut amqo_config = amqo_config;
        amqo_config.connection = crate::Connection::Data {
            host: host.to_string(),
            port: amqp_host_port,
            username: username,
            password: password,
        };
        (container, amqo_config)
    }

    use crate::events::ev_1::Ev1;
    use crate::events::ev_2::Ev2;

    #[tokio::test]
    async fn content_processor_with_subscriber_simple() {
        trace_init();
        let (_container, amqo_config) = start_rabbitmq_container().await;
        let mut processor = Arc::new(ContentProcessor::new());

        Arc::get_mut(&mut processor).unwrap().register::<Ev1>();
        Arc::get_mut(&mut processor).unwrap().register::<Ev2>();

        let event_processor1 = Ev1::dispatcher();
        let (mut rx1, mut unsubsriber1) = event_processor1.write().await.add_channel(None).await;

        let event_processor2 = Ev2::dispatcher();
        let (mut rx2, mut unsubsriber2) = event_processor2.write().await.add_channel(None).await;

        let consumer = TestConsumer::new(Arc::clone(&processor));

        let event_bus = MqEventBus::new_from_config(consumer, amqo_config, "test").await.unwrap();

        let msg1 = Ev1 {
            data: "test".to_string(),
            buyer_identity_guid: "test".to_string(),
        };
        let msg2 = Ev2 {
            data: "test2".to_string(),
            buyer_identity_guid: "test2".to_string(),
        };

        event_bus.publish(msg1).await.unwrap();
        event_bus.publish(msg2).await.unwrap();

        let result = rx1.recv().await;
        assert!(result.is_some(), "result is not some {} ", Ev1::key());
        let result = rx2.recv().await;
        assert!(result.is_some(), "result is not some {} ", Ev2::key());

        unsubsriber1.unsubscribe().await;
        unsubsriber2.unsubscribe().await;
        event_bus.stop().await.unwrap();
    }

    #[tokio::test]
    async fn content_processor_with_subscriber_with_added_removed_channels() {
        let (_container, amqo_config) = start_rabbitmq_container().await;
        let mut processor = Arc::new(ContentProcessor::new());

        Arc::get_mut(&mut processor).unwrap().register::<Ev1>();
        Arc::get_mut(&mut processor).unwrap().register::<Ev2>();

        let (mut rx1, mut unsubsriber1) = Ev1::dispatcher().write().await.add_channel(None).await;

        let (mut rx2, mut unsubsriber2) = Ev2::dispatcher().write().await.add_channel(None).await;

        let consumer = TestConsumer::new(Arc::clone(&processor));

        let event_bus = MqEventBus::new_from_config(consumer, amqo_config, "test").await.unwrap();

        let msg1 = Ev1 {
            data: "test".to_string(),
            buyer_identity_guid: "test".to_string(),
        };
        let msg2 = Ev2 {
            data: "test2".to_string(),
            buyer_identity_guid: "test2".to_string(),
        };

        event_bus.publish(msg1).await.unwrap();
        event_bus.publish(msg2).await.unwrap();

        let result = rx1.recv().await;
        assert!(result.is_some(), "result is not some {} ", Ev1::key());
        let result = rx2.recv().await;
        assert!(result.is_some(), "result is not some {} ", Ev2::key());

        unsubsriber1.unsubscribe().await;
        unsubsriber2.unsubscribe().await;
        event_bus.stop().await.unwrap();
    }

    #[tokio::test]
    async fn content_processor_with_subscriber_and_filter() {
        let (_container, amqo_config) = start_rabbitmq_container().await;
        let mut processor = Arc::new(ContentProcessor::new());

        Arc::get_mut(&mut processor).unwrap().register::<Ev1>();
        Arc::get_mut(&mut processor).unwrap().register::<Ev2>();

        let (mut rx1, mut unsubsriber1) = Ev1::dispatcher().write().await.add_channel(Some(Box::new(|ev: &Ev1| ev.data == "test"))).await;

        let (mut rx2, mut unsubsriber2) = Ev2::dispatcher().write().await.add_channel(None).await;

        let consumer = TestConsumer::new(Arc::clone(&processor));

        let event_bus = MqEventBus::new_from_config(consumer, amqo_config, "test").await.unwrap();

        let msg1 = Ev1 {
            data: "test should not be received".to_string(),
            buyer_identity_guid: "test".to_string(),
        };
        let msg2 = Ev2 {
            data: "test2".to_string(),
            buyer_identity_guid: "test2".to_string(),
        };

        event_bus.publish(msg1).await.unwrap();
        event_bus.publish(msg2).await.unwrap();

        let result = rx1.is_empty();
        if !result {
            let result = rx1.recv().await;
            println!("result  should be empty{} {:#?} ", Ev1::key(), result.unwrap());
        }
        let result = rx2.recv().await;
        assert!(result.is_some(), "result is not some {} ", Ev2::key());
        unsubsriber1.unsubscribe().await;
        unsubsriber2.unsubscribe().await;
        event_bus.stop().await.unwrap();
    }

    #[tokio::test]
    async fn content_processor_with_different_publisher_consumer() {
        trace_init();

        let (_container, amqo_config) = start_rabbitmq_container().await;

        // publisher
        let e_pub = MqEventBus::new_from_config_publisher(vec![Ev1::key(), Ev2::key()], amqo_config.clone()).await.unwrap();

        let msg1 = Ev1 {
            data: "test".to_string(),
            buyer_identity_guid: "test".to_string(),
        };
        let msg2 = Ev2 {
            data: "test2".to_string(),
            buyer_identity_guid: "test2".to_string(),
        };

        e_pub.publish(msg1).await.unwrap();
        e_pub.publish(msg2).await.unwrap();

        e_pub.stop().await.unwrap();

        // consumer
        let mut processor = Arc::new(ContentProcessor::new());

        Arc::get_mut(&mut processor).unwrap().register::<Ev1>();
        Arc::get_mut(&mut processor).unwrap().register::<Ev2>();

        let (mut rx1, mut unsuscriber1) = Ev1::dispatcher().write().await.add_channel(None).await;

        let (mut rx2, mut unsuscriber2) = Ev2::dispatcher().write().await.add_channel(None).await;

        let consumer = TestConsumer::new(Arc::clone(&processor));

        let event_bus = MqEventBus::new_from_config(consumer, amqo_config, "test").await.unwrap();

        let result = rx1.recv().await;
        assert!(result.is_some(), "result is not some {} ", Ev1::key());
        let result = rx2.recv().await;
        assert!(result.is_some(), "result is not some {} ", Ev2::key());

        unsuscriber1.unsubscribe().await;
        unsuscriber2.unsubscribe().await;

        event_bus.stop().await.unwrap();
    }
}
