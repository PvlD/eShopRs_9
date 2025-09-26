use std::sync::Arc;

use bollard::Docker;
use bollard::query_parameters::ListContainersOptions;
use rabbit_mq_bus::{AMQOConfig, Connection, ContentProcessor, EventBusFactory, MqEventBus};

use super::Consumer;

pub async fn init_eventbus(consumer_tag: &str) -> MqEventBus {
    let amqo_config = AMQOConfig::from_env();

    let connection = match amqo_config.connection {
        Connection::Data { host, port: _, username, password } => {
            let port = get_eventbus_mammed_port().await;
            println!("mapped docker port: {}", port);
            Connection::Data { host, port, username, password }
        }
        Connection::String(val) => Connection::String(val),
    };

    let amqo_config = AMQOConfig { connection: connection, ..amqo_config };

    let mut processor: Arc<ContentProcessor> = Arc::new(ContentProcessor::new());
    app_events::integration_events::register(&mut processor);

    let consumer = Consumer::new(Arc::clone(&processor));
    let eventbus = MqEventBus::new_from_config(consumer, amqo_config, consumer_tag).await.unwrap();
    eventbus
}

async fn get_eventbus_mammed_port() -> u16 {
    let docker = Docker::connect_with_local_defaults().unwrap();

    let options = ListContainersOptions { all: true, ..Default::default() };
    let containers = docker.list_containers(Some(options)).await.unwrap();

    for container in containers {
        if container.names.as_ref().unwrap().join(", ").contains("eventbus-") {
            println!("container: {}", container.names.as_ref().unwrap().join(", "));

            if let Some(ports) = container.ports {
                for port in ports {
                    println!("public_port: {} private_port: {}", port.public_port.unwrap(), port.private_port);
                    match (port.ip, port.public_port, port.private_port) {
                        (Some(_ip), Some(public_port), private_port) if private_port == 5672 => {
                            return public_port;
                        }
                        (_, _, _) => {}
                    }
                }
            }
            panic!("no port mapping found with eventbus")
        }
    }

    panic!("no container found with eventbus")
}
