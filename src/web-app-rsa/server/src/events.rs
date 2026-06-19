use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use eventbus::IntegrationEvent;

macro_rules! define_event {
    ($name:ident, $event_name:expr) => {
        #[derive(Debug, Clone, Serialize, Deserialize)]
        #[serde(rename_all = "PascalCase")]
        pub struct $name {
            pub id: Uuid,
            pub creation_date: DateTime<Utc>,
            pub order_id: i32,
            pub order_status: String,
            pub buyer_name: String,
            pub buyer_identity_guid: String,
        }

        impl IntegrationEvent for $name {
            fn event_name() -> &'static str {
                $event_name
            }

            fn id(&self) -> Uuid {
                self.id
            }

            fn creation_date(&self) -> DateTime<Utc> {
                self.creation_date
            }

            fn buyer_id(&self) -> &str {
                &self.buyer_identity_guid
            }
        }
    };
}

define_event!(
    OrderStatusChangedToSubmittedIntegrationEvent,
    "OrderStatusChangedToSubmittedIntegrationEvent"
);

define_event!(
    OrderStatusChangedToAwaitingValidationIntegrationEvent,
    "OrderStatusChangedToAwaitingValidationIntegrationEvent"
);

define_event!(
    OrderStatusChangedToStockConfirmedIntegrationEvent,
    "OrderStatusChangedToStockConfirmedIntegrationEvent"
);

define_event!(
    OrderStatusChangedToPaidIntegrationEvent,
    "OrderStatusChangedToPaidIntegrationEvent"
);

define_event!(
    OrderStatusChangedToShippedIntegrationEvent,
    "OrderStatusChangedToShippedIntegrationEvent"
);

define_event!(
    OrderStatusChangedToCancelledIntegrationEvent,
    "OrderStatusChangedToCancelledIntegrationEvent"
);
