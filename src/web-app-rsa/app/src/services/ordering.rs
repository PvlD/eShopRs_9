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

#[server]
pub async fn get_orders() -> Result<Vec<OrderRecord>, AppError> {
    use std::sync::Arc;
    use settings::Settings;
    use leptos::prelude::expect_context;
    use tower_sessions::Session;
    use leptos_axum::extract;

    let settings = expect_context::<Arc<Settings>>();
    let base = settings.services.ordering_api.http.clone();
    let version = settings.services.ordering_api.version.clone().unwrap();
    let url = format!("{}/api/Orders/?api-version={}", base, version);

    let session: Session = extract().await.map_err(to_server_err)?;
    let access_token: Option<String> = session.get("access_token").await.map_err(to_server_err)?;
    let token = access_token.ok_or_else(auth_err)?;

    let client = reqwest::Client::new();
    let resp = client
        .get(&url)
        .bearer_auth(token)
        .send()
        .await
        .map_err(to_server_err)?;
    let orders: Vec<OrderRecord> = resp.json().await.map_err(to_server_err)?;
    Ok(orders)
}

#[server]
pub async fn create_order(request: CreateOrderRequest) -> Result<(), AppError> {
    use std::sync::Arc;
    use settings::Settings;
    use leptos::prelude::expect_context;
    use tower_sessions::Session;
    use leptos_axum::extract;
    use uuid::Uuid;

    let settings = expect_context::<Arc<Settings>>();
    let base = settings.services.ordering_api.http.clone();
    let version = settings.services.ordering_api.version.clone().unwrap();
    let url = format!("{}/api/Orders/?api-version={}", base, version);

    let session: Session = extract().await.map_err(to_server_err)?;
    let access_token: Option<String> = session.get("access_token").await.map_err(to_server_err)?;
    let token = access_token.ok_or_else(auth_err)?;

    let request_id = Uuid::new_v4();

    let client = reqwest::Client::new();
    client
        .post(&url)
        .bearer_auth(token)
        .header("x-requestid", request_id.to_string())
        .json(&request)
        .send()
        .await
        .map_err(to_server_err)?
        .error_for_status()
        .map_err(to_server_err)?;
    Ok(())
}
