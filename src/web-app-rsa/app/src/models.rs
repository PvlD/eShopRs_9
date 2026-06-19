use garde::Validate;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CatalogItem {
    pub id: i32,
    pub name: String,
    pub description: String,
    pub price: f64,
    pub catalog_brand_id: i32,
    pub catalog_brand: Option<CatalogBrand>,
    pub catalog_type_id: i32,
    pub catalog_type: Option<CatalogItemType>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CatalogResult {
    pub page_index: i32,
    pub page_size: i32,
    pub count: i32,
    pub data: Vec<CatalogItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CatalogBrand {
    pub id: i32,
    pub brand: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CatalogItemType {
    pub id: i32,
    #[serde(rename = "type")]
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BasketQuantity {
    pub product_id: i32,
    pub quantity: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderRecord {
    pub order_number: i32,
    pub date: String,
    pub status: String,
    pub total: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BasketItem {
    pub id: String,
    pub product_id: i32,
    pub product_name: String,
    pub unit_price: f64,
    pub old_unit_price: f64,
    pub quantity: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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
    pub card_expiration: String,
    pub card_security_number: String,
    pub card_type_id: i32,
    pub buyer: String,
    pub items: Vec<BasketItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct BasketCheckoutInfo {
    #[garde(length(min = 1))]
    pub street: String,
    #[garde(length(min = 1))]
    pub city: String,
    #[garde(length(min = 1))]
    pub state: String,
    #[garde(length(min = 1))]
    pub country: String,
    #[garde(length(min = 1))]
    pub zip_code: String,
    #[garde(skip)]
    pub card_number: Option<String>,
    #[garde(skip)]
    pub card_holder_name: Option<String>,
    #[garde(skip)]
    pub card_security_number: Option<String>,
    #[garde(skip)]
    pub card_expiration: Option<String>,
    #[garde(skip)]
    pub card_type_id: i32,
    #[garde(skip)]
    pub buyer: Option<String>,
    #[garde(skip)]
    pub request_id: String,
}

impl BasketCheckoutInfo {
    pub fn field_label(field: &str) -> &'static str {
        match field {
            "street" => "Street",
            "city" => "City",
            "state" => "State",
            "zip_code" => "Zip Code",
            "country" => "Country",
            _ => panic!("unknown field: {field}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_validation_fields_have_labels() {
        for field in ["street", "city", "state", "zip_code", "country"] {
            BasketCheckoutInfo::field_label(field);
        }
    }
}
