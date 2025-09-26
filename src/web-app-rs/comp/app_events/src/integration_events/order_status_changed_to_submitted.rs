use rabbit_mq_bus::{Content, Dispatcherable, FromContent, Keyed};
use serde::{Deserialize, Serialize};

use rabbit_mq_bus::ebus;

use crate::integration_events::IntegrationEvent;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct OrderStatusChangedToSubmitted {
    #[serde(flatten)]
    pub base: IntegrationEvent,
    pub order_id: i32,
    pub order_status: String,
    pub buyer_name: String,
    pub buyer_identity_guid: String,
}

impl Default for OrderStatusChangedToSubmitted {
    fn default() -> Self {
        Self::new()
    }
}

impl OrderStatusChangedToSubmitted {
    pub fn new() -> OrderStatusChangedToSubmitted {
        OrderStatusChangedToSubmitted {
            base: IntegrationEvent::new(),
            order_id: 0,
            order_status: "".to_string(),
            buyer_name: "".to_string(),
            buyer_identity_guid: "".to_string(),
        }
    }
}
impl Keyed for OrderStatusChangedToSubmitted {
    fn key() -> &'static str {
        "OrderStatusChangedToSubmittedIntegrationEvent"
    }
}

impl FromContent for OrderStatusChangedToSubmitted {
    fn from_content(data: Vec<u8>) -> Result<Self, ebus::lib_err::AppError> {
        let event = serde_json::from_slice(&data)?;
        Ok(event)
    }
}

impl Content for OrderStatusChangedToSubmitted {
    fn content(&self) -> Result<(&str, Vec<u8>), ebus::lib_err::AppError> {
        let json = serde_json::to_vec(&self)?;
        Ok((OrderStatusChangedToSubmitted::key(), json))
    }
}

impl Dispatcherable<OrderStatusChangedToSubmitted> for OrderStatusChangedToSubmitted {
    fn dispatcher() -> Arc<RwLock<Dispatcher<OrderStatusChangedToSubmitted>>> {
        get_dispatcher().clone()
    }
}

use rabbit_mq_bus::Dispatcher;
use std::sync::{Arc, OnceLock};
use tokio::sync::RwLock;
static DISPATCHER: OnceLock<Arc<RwLock<Dispatcher<OrderStatusChangedToSubmitted>>>> = OnceLock::new();

fn get_dispatcher() -> &'static Arc<RwLock<Dispatcher<OrderStatusChangedToSubmitted>>> {
    DISPATCHER.get_or_init(|| Arc::new(RwLock::new(Dispatcher::new())))
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;
    use uuid::Uuid;

    use super::*;

    #[test]
    fn test_from_content() {
        let content = r#"{"OrderId":12,"OrderStatus":"StockConfirmed","BuyerName":"Bob","BuyerIdentityGuid":"f3db6221-7a25-4f03-b363-d7654556a7c9","Id":"c8168f83-42d2-483c-b217-01f7eb87ccfb","CreationDate":"2025-08-09T20:51:51.3865279Z"}"#;

        let event = OrderStatusChangedToSubmitted::from_content(content.as_bytes().to_vec()).unwrap();
        assert_eq!(event.order_id, 12);
        assert_eq!(event.order_status, "StockConfirmed");
        assert_eq!(event.buyer_name, "Bob");
        assert_eq!(event.buyer_identity_guid, "f3db6221-7a25-4f03-b363-d7654556a7c9");
        assert_eq!(event.base.id, Uuid::from_str("c8168f83-42d2-483c-b217-01f7eb87ccfb").unwrap());
    }
}
