#[derive(Clone)]
pub enum Connection {
    Data { host: String, port: u16, username: String, password: String },
    String(String),
}

#[derive(Clone)]
pub struct AMQOConfig {
    pub connection: Connection,
    pub exchange_name: String,
    pub queue_name: String,
}

impl AMQOConfig {
    pub fn new_data(host: String, port: u16, username: String, password: String, exchange_name: String, queue_name: String) -> Self {
        Self {
            connection: Connection::Data { host, port, username, password },
            exchange_name,
            queue_name,
        }
    }

    pub fn new_connection_string(connection_string: String, exchange_name: Option<String>, queue_name: Option<String>) -> Self {
        Self {
            connection: Connection::String(connection_string),
            exchange_name: exchange_name.unwrap_or("".to_string()),
            queue_name: queue_name.unwrap_or("".to_string()),
        }
    }

    pub fn from_env() -> Self {
        let exchange_name = std::env::var("AMQP_EXCHANGE_NAME").ok();
        let queue_name = std::env::var("AMQP_QUEUE_NAME").ok();

        let connection_var = "ConnectionStrings__eventbus";

        if let Ok(connection) = std::env::var(connection_var) {
            return Self {
                connection: Connection::String(connection),
                exchange_name: exchange_name.unwrap_or("".to_string()),
                queue_name: queue_name.unwrap_or("".to_string()),
            };
        }

        Self {
            connection: Connection::Data {
                host: std::env::var("AMQP_HOST").unwrap(),
                port: std::env::var("AMQP_PORT").unwrap().parse().unwrap(),
                username: std::env::var("AMQP_USERNAME").unwrap(),
                password: std::env::var("AMQP_PASSWORD").unwrap(),
            },
            exchange_name: exchange_name.unwrap_or("".to_string()),
            queue_name: queue_name.unwrap_or("".to_string()),
        }
    }
}
