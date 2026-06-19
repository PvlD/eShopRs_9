use crate::models::BasketQuantity;
use leptos::server;
use crate::error::AppError;

#[cfg(feature = "ssr")]
pub(crate) mod basket_grpc {
    tonic::include_proto!("basket_api");
}

#[cfg_attr(not(feature = "ssr"), allow(dead_code))]
fn to_server_err(e: impl std::fmt::Display) -> AppError {
    AppError(e.to_string())
}

#[cfg_attr(not(feature = "ssr"), allow(dead_code))]
fn auth_err() -> AppError {
    AppError("Not authenticated".into())
}

#[server]
pub async fn get_basket() -> Result<Vec<BasketQuantity>, AppError> {
    use std::sync::Arc;
    use settings::Settings;
    use leptos::prelude::expect_context;
    use tower_sessions::Session;
    use leptos_axum::extract;
    use tonic::IntoRequest;

    let settings = expect_context::<Arc<Settings>>();
    let url = settings.services.basket_api.http.clone();

    let session: Session = extract().await.map_err(to_server_err)?;
    let access_token: Option<String> = session.get("access_token").await.map_err(to_server_err)?;
    let token = access_token.ok_or_else(auth_err)?;

    let mut client = basket_grpc::basket_client::BasketClient::connect(url)
        .await
        .map_err(to_server_err)?;
    let mut request = basket_grpc::GetBasketRequest {}.into_request();
    request
        .metadata_mut()
        .insert("authorization", format!("Bearer {token}").parse().unwrap());
    let response = client.get_basket(request).await.map_err(to_server_err)?;

    Ok(response
        .into_inner()
        .items
        .into_iter()
        .map(|item| BasketQuantity {
            product_id: item.product_id,
            quantity: item.quantity,
        })
        .collect())
}

#[server]
pub async fn update_basket(items: Vec<BasketQuantity>) -> Result<(), AppError> {
    use std::sync::Arc;
    use settings::Settings;
    use leptos::prelude::expect_context;
    use tower_sessions::Session;
    use leptos_axum::extract;
    use tonic::IntoRequest;

    let settings = expect_context::<Arc<Settings>>();
    let url = settings.services.basket_api.http.clone();

    let session: Session = extract().await.map_err(to_server_err)?;
    let access_token: Option<String> = session.get("access_token").await.map_err(to_server_err)?;
    let token = access_token.ok_or_else(auth_err)?;

    let items = items
        .into_iter()
        .map(|item| basket_grpc::BasketItem {
            product_id: item.product_id,
            quantity: item.quantity,
        })
        .collect();

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
pub async fn delete_basket() -> Result<(), AppError> {
    use std::sync::Arc;
    use settings::Settings;
    use leptos::prelude::expect_context;
    use tower_sessions::Session;
    use leptos_axum::extract;
    use tonic::IntoRequest;

    let settings = expect_context::<Arc<Settings>>();
    let url = settings.services.basket_api.http.clone();

    let session: Session = extract().await.map_err(to_server_err)?;
    let access_token: Option<String> = session.get("access_token").await.map_err(to_server_err)?;
    let token = access_token.ok_or_else(auth_err)?;

    let mut client = basket_grpc::basket_client::BasketClient::connect(url)
        .await
        .map_err(to_server_err)?;
    let mut request = basket_grpc::DeleteBasketRequest {}.into_request();
    request
        .metadata_mut()
        .insert("authorization", format!("Bearer {token}").parse().unwrap());
    client.delete_basket(request).await.map_err(to_server_err)?;

    Ok(())
}
