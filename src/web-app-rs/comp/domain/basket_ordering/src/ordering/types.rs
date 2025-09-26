use rust_decimal::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Order {
    pub order_number: usize,
    pub date: chrono::DateTime<chrono::Utc>,
    pub status: String,
    pub total: Decimal,
}

impl Order {
    pub fn new() -> Self {
        Self {
            order_number: Default::default(),
            date: chrono::DateTime::default(),
            status: String::new(),
            total: Decimal::zero(),
        }
    }
}
