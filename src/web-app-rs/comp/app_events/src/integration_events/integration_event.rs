use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct IntegrationEvent {
    pub id: Uuid,
    pub creation_date: DateTime<Utc>,
}

impl Default for IntegrationEvent {
    fn default() -> Self {
        Self::new()
    }
}

impl IntegrationEvent {
    pub fn new() -> IntegrationEvent {
        IntegrationEvent { id: Uuid::new_v4(), creation_date: Utc::now() }
    }
}
