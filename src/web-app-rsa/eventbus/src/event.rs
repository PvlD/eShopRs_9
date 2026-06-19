use chrono::{DateTime, Utc};
use serde::de::DeserializeOwned;
use serde::Serialize;
use uuid::Uuid;

pub trait IntegrationEvent: DeserializeOwned + Serialize + Send + Sync + 'static {
    fn event_name() -> &'static str;
    fn id(&self) -> Uuid;
    fn creation_date(&self) -> DateTime<Utc>;
    fn buyer_id(&self) -> &str;
}
