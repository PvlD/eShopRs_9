use crate::models::*;
use leptos::server;
use crate::error::AppError;

#[cfg_attr(not(feature = "ssr"), allow(dead_code))]
fn to_server_err(e: impl std::fmt::Display) -> AppError {
    AppError(e.to_string())
}

#[cfg_attr(not(feature = "ssr"), allow(dead_code))]
fn auth_err() -> AppError {
    AppError("Not authenticated".into())
}

#[cfg(feature = "ssr")]
fn catalog_base() -> String {
    use std::sync::Arc;
    use settings::Settings;
    use leptos::prelude::expect_context;
    expect_context::<Arc<Settings>>()
        .services
        .catalog_api
        .http
        .clone()
}

#[cfg(feature = "ssr")]
fn catalog_api_version() -> String {
    use std::sync::Arc;
    use settings::Settings;
    use leptos::prelude::expect_context;
    expect_context::<Arc<Settings>>()
        .services
        .catalog_api
        .version
        .clone()
        .unwrap()
}

#[cfg(feature = "ssr")]
fn basket_url() -> String {
    use std::sync::Arc;
    use settings::Settings;
    use leptos::prelude::expect_context;
    expect_context::<Arc<Settings>>()
        .services
        .basket_api
        .http
        .clone()
}

#[server]
pub async fn get_basket_items() -> Result<Vec<BasketItem>, AppError> {
    use tower_sessions::Session;
    use leptos_axum::extract;
    use tonic::IntoRequest;

    let session: Session = extract().await.map_err(to_server_err)?;
    let access_token: Option<String> = session.get("access_token").await.map_err(to_server_err)?;
    let token = match access_token {
        Some(t) => t,
        None => return Ok(vec![]),
    };

    use crate::services::basket::basket_grpc;
    let url = basket_url();
    let mut client = basket_grpc::basket_client::BasketClient::connect(url)
        .await
        .map_err(to_server_err)?;
    let mut request = basket_grpc::GetBasketRequest {}.into_request();
    request
        .metadata_mut()
        .insert("authorization", format!("Bearer {token}").parse().unwrap());
    let response = client.get_basket(request).await.map_err(to_server_err)?;
    let quantities = response.into_inner().items;

    if quantities.is_empty() {
        return Ok(vec![]);
    }

    let base = catalog_base();
    let version = catalog_api_version();
    let ids: Vec<String> = quantities.iter().map(|i| i.product_id.to_string()).collect();
    let url = format!(
        "{}/api/catalog/items/by?ids={}&api-version={}",
        base,
        ids.join("&ids="),
        version
    );
    let client = reqwest::Client::new();
    let resp = client.get(&url).send().await.map_err(to_server_err)?;
    let catalog_items: Vec<CatalogItem> = resp.json().await.map_err(to_server_err)?;

    use std::collections::HashMap;
    let catalog_map: HashMap<i32, &CatalogItem> =
        catalog_items.iter().map(|item| (item.id, item)).collect();

    let basket_items: Vec<BasketItem> = quantities
        .into_iter()
        .filter_map(|q| {
            catalog_map.get(&q.product_id).map(|cat| BasketItem {
                id: uuid::Uuid::new_v4().to_string(),
                product_id: cat.id,
                product_name: cat.name.clone(),
                unit_price: cat.price,
                old_unit_price: 0.0,
                quantity: q.quantity,
            })
        })
        .collect();

    Ok(basket_items)
}

#[server]
pub async fn add_basket_item(product_id: i32) -> Result<(), AppError> {
    use tower_sessions::Session;
    use leptos_axum::extract;
    use tonic::IntoRequest;

    let session: Session = extract().await.map_err(to_server_err)?;
    let access_token: Option<String> = session.get("access_token").await.map_err(to_server_err)?;
    let token = access_token.ok_or_else(auth_err)?;

    use crate::services::basket::basket_grpc;
    let url = basket_url();
    let mut client = basket_grpc::basket_client::BasketClient::connect(url.clone())
        .await
        .map_err(to_server_err)?;
    let mut request = basket_grpc::GetBasketRequest {}.into_request();
    request
        .metadata_mut()
        .insert("authorization", format!("Bearer {token}").parse().unwrap());
    let response = client.get_basket(request).await.map_err(to_server_err)?;
    let mut items = response.into_inner().items;

    if let Some(existing) = items.iter_mut().find(|i| i.product_id == product_id) {
        existing.quantity += 1;
    } else {
        items.push(basket_grpc::BasketItem {
            product_id,
            quantity: 1,
        });
    }

    let mut client = basket_grpc::basket_client::BasketClient::connect(url)
        .await
        .map_err(to_server_err)?;
    let mut request = basket_grpc::UpdateBasketRequest { items }.into_request();
    request
        .metadata_mut()
        .insert("authorization", format!("Bearer {token}").parse().unwrap());
    client.update_basket(request).await.map_err(to_server_err)?;

    Ok(())
}

#[server]
pub async fn set_basket_quantity(product_id: i32, quantity: i32) -> Result<(), AppError> {
    use tower_sessions::Session;
    use leptos_axum::extract;
    use tonic::IntoRequest;

    let session: Session = extract().await.map_err(to_server_err)?;
    let access_token: Option<String> = session.get("access_token").await.map_err(to_server_err)?;
    let token = access_token.ok_or_else(auth_err)?;

    use crate::services::basket::basket_grpc;
    let url = basket_url();
    let mut client = basket_grpc::basket_client::BasketClient::connect(url.clone())
        .await
        .map_err(to_server_err)?;
    let mut request = basket_grpc::GetBasketRequest {}.into_request();
    request
        .metadata_mut()
        .insert("authorization", format!("Bearer {token}").parse().unwrap());
    let response = client.get_basket(request).await.map_err(to_server_err)?;
    let mut items = response.into_inner().items;

    let changed = if let Some(existing) = items.iter_mut().find(|i| i.product_id == product_id) {
        if quantity > 0 {
            existing.quantity = quantity;
        } else {
            items.retain(|i| i.product_id != product_id);
        }
        true
    } else if quantity > 0 {
        items.push(basket_grpc::BasketItem {
            product_id,
            quantity,
        });
        true
    } else {
        false
    };

    if changed {
        let mut client = basket_grpc::basket_client::BasketClient::connect(url)
            .await
            .map_err(to_server_err)?;
        let mut request = basket_grpc::UpdateBasketRequest { items }.into_request();
        request
            .metadata_mut()
            .insert("authorization", format!("Bearer {token}").parse().unwrap());
        client.update_basket(request).await.map_err(to_server_err)?;
    }

    Ok(())
}

#[server]
pub async fn checkout(info: BasketCheckoutInfo) -> Result<(), AppError> {
    use std::sync::Arc;
    use settings::Settings;
    use leptos::prelude::expect_context;
    use tower_sessions::Session;
    use leptos_axum::extract;
    use tonic::IntoRequest;

    let settings = expect_context::<Arc<Settings>>();

    let session: Session = extract().await.map_err(to_server_err)?;
    let access_token: Option<String> = session.get("access_token").await.map_err(to_server_err)?;
    let token = access_token.ok_or_else(auth_err)?;
    let user_name: Option<String> = session.get("user_name").await.map_err(to_server_err)?;
    let user_name = user_name.unwrap_or_default();

    let buyer_id = extract_sub_from_access_token(&token)?;

    let order_items = get_basket_items().await?;
    if order_items.is_empty() {
        return Err(AppError("Basket is empty".into()));
    }

    let request_id = if info.request_id.is_empty() {
        uuid::Uuid::new_v4().to_string()
    } else {
        info.request_id
    };

    let order = CreateOrderRequest {
        user_id: buyer_id.clone(),
        user_name,
        city: info.city,
        street: info.street,
        state: info.state,
        country: info.country,
        zip_code: info.zip_code,
        card_number: info.card_number.unwrap_or_else(|| "1111222233334444".to_string()),
        card_holder_name: info.card_holder_name.unwrap_or_else(|| "TESTUSER".to_string()),
        card_expiration: info.card_expiration.unwrap_or_else(|| {
            chrono::Utc::now()
                .checked_add_signed(chrono::Duration::days(365))
                .map(|d| d.format("%Y-%m-%dT%H:%M:%SZ").to_string())
                .unwrap_or_default()
        }),
        card_security_number: info.card_security_number.unwrap_or_else(|| "111".to_string()),
        card_type_id: info.card_type_id,
        buyer: buyer_id,
        items: order_items,
    };

    let ordering_url = settings.services.ordering_api.http.clone();
    let version = settings.services.ordering_api.version.clone().unwrap();
    let url = format!("{}/api/Orders/?api-version={}", ordering_url, version);

    let http_client = reqwest::Client::new();
    http_client
        .post(&url)
        .bearer_auth(&token)
        .header("x-requestid", &request_id)
        .json(&order)
        .send()
        .await
        .map_err(to_server_err)?
        .error_for_status()
        .map_err(to_server_err)?;

    use crate::services::basket::basket_grpc;
    let basket_url = basket_url();
    let mut client = basket_grpc::basket_client::BasketClient::connect(basket_url.clone())
        .await
        .map_err(to_server_err)?;
    let mut request = basket_grpc::DeleteBasketRequest {}.into_request();
    request
        .metadata_mut()
        .insert("authorization", format!("Bearer {token}").parse().unwrap());
    client.delete_basket(request).await.map_err(to_server_err)?;

    Ok(())
}

#[cfg(feature = "ssr")]
fn extract_sub_from_access_token(access_token: &str) -> Result<String, AppError> {
    let parts: Vec<&str> = access_token.split('.').collect();
    let payload_b64 = parts
        .get(1)
        .ok_or_else(|| to_server_err("Invalid access token format"))?;
    let decoded = base64_url_decode(payload_b64)?;
    let payload_str = String::from_utf8(decoded).map_err(to_server_err)?;
    let sub_key = "\"sub\":\"";
    if let Some(start) = payload_str.find(sub_key) {
        let rest = &payload_str[start + sub_key.len()..];
        if let Some(end) = rest.find('"') {
            return Ok(rest[..end].to_string());
        }
    }
    Err(to_server_err("No sub claim in access token"))
}

#[server]
pub async fn get_checkout_info() -> Result<BasketCheckoutInfo, AppError> {
    use tower_sessions::Session;
    use leptos_axum::extract;
    let session: Session = extract().await.map_err(to_server_err)?;
    Ok(BasketCheckoutInfo {
        street: session.get("address_street").await.map_err(to_server_err)?.unwrap_or_default(),
        city: session.get("address_city").await.map_err(to_server_err)?.unwrap_or_default(),
        state: session.get("address_state").await.map_err(to_server_err)?.unwrap_or_default(),
        country: session.get("address_country").await.map_err(to_server_err)?.unwrap_or_default(),
        zip_code: session.get("address_zip_code").await.map_err(to_server_err)?.unwrap_or_default(),
        card_number: None,
        card_holder_name: None,
        card_security_number: None,
        card_expiration: None,
        card_type_id: 1,
        buyer: None,
        request_id: String::new(),
    })
}

#[cfg(feature = "ssr")]
fn base64_url_decode(input: &str) -> Result<Vec<u8>, AppError> {
    use base64::Engine;
    let standard = input.replace('-', "+").replace('_', "/");
    let padded = match standard.len() % 4 {
        0 => standard,
        n => {
            let mut s = standard;
            for _ in 0..(4 - n) {
                s.push('=');
            }
            s
        }
    };
    base64::engine::general_purpose::STANDARD
        .decode(padded.as_bytes())
        .map_err(to_server_err)
}
