use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateOrderRequest {
    pub user_id: String,
    pub user_name: String,
    pub city: String,
    pub street: String,
    pub state: String,
    pub country: String,
    pub zip_code: String,
    pub card_number: String,
    pub card_holder_name: String,
    pub card_expiration: chrono::DateTime<chrono::Utc>,
    pub card_security_number: String,
    pub card_type_id: i32,
    pub buyer: String,
    pub items: Vec<BasketItem>,
}

impl CreateOrderRequest {
    pub fn new() -> Self {
        Self {
            user_id: String::default(),
            user_name: String::default(),
            city: String::default(),
            street: String::default(),
            state: String::default(),
            country: String::default(),
            zip_code: String::default(),
            card_number: String::default(),
            card_holder_name: String::default(),
            card_expiration: chrono::DateTime::<chrono::Utc>::default(),
            card_security_number: String::default(),
            card_type_id: i32::default(),
            buyer: String::default(),
            items: Vec::<BasketItem>::default(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BasketItem {
    pub id: String,
    pub product_id: i32,
    pub product_name: String,
    pub unit_price: Decimal,
    pub old_unit_price: Decimal,
    pub quantity: i32,
}

impl BasketItem {
    pub fn new() -> Self {
        Self {
            id: String::default(),
            product_id: i32::default(),
            product_name: String::default(),
            unit_price: Decimal::default(),
            old_unit_price: Decimal::default(),
            quantity: i32::default(),
        }
    }
}
