use leptos::{
    server,
    server_fn::{BoxedStream, ServerFnError, Websocket, codec::JsonEncoding},
};

use crate::basket::types::BasketQuantity;

#[cfg(feature = "ssr")]
use leptos::prelude::expect_context;

#[cfg(feature = "ssr")]
use crate::basket::service::BasketServiceContext;

#[server]
#[middleware(auth::RequireAuth)]
pub async fn get_basket() -> Result<Vec<BasketQuantity>, crate::AppError> {
    let basket_service_context: BasketServiceContext = expect_context();

    basket_service_context.service.get_basket().await
}

#[server]
#[middleware(auth::RequireAuth)]
pub async fn update_basket(items: Vec<BasketQuantity>) -> Result<(), crate::AppError> {
    let basket_service_context: BasketServiceContext = expect_context();

    basket_service_context.service.update_basket(items).await
}

#[server]
#[middleware(auth::RequireAuth)]
pub async fn delete_basket() -> Result<(), crate::AppError> {
    let basket_service_context: BasketServiceContext = expect_context();

    basket_service_context.service.delete_basket().await
}

#[server(protocol = Websocket<JsonEncoding, JsonEncoding>)]
#[middleware(auth::RequireAuth)]
pub async fn basket_state_notify(input_: BoxedStream<bool, ServerFnError>) -> Result<BoxedStream<bool, ServerFnError>, ServerFnError> {
    use app_events::eg_by_id_filter;
    use futures::channel::mpsc;
    use leptos_axum::extract;

    use axum::http::Extensions;

    let extensions: Extensions = extract().await?;
    let auth_session = extensions.get::<auth::users::AuthSession>().ok_or(crate::AppError::Unauthorized)?;
    let user_id = auth_session.user.as_ref().ok_or(crate::AppError::Unauthorized)?.sub.clone();
    let mut input = input_;

    // create a channel of outgoing websocket messages
    // we'll return rx, so sending a message to tx will send a message to the client via the websocket
    let (mut tx, rx) = mpsc::channel::<Result<bool, ServerFnError>>(1);

    tokio::spawn(async move {
        use futures::{SinkExt, StreamExt, stream::FuturesUnordered};
        use std::{future::Future, pin::Pin};

        let (mut rx_evt, mut unsubscribe) = eg_by_id_filter::register_group(Some(user_id)).await;

        let mut futures = FuturesUnordered::new();

        const MAX_SEND_COUNT: usize = 10;
        const MAX_SEND_TIMEOUT: u64 = 1000;

        futures.push(Box::pin(async move {
            'app_events: while let Some(_event) = rx_evt.recv().await {
                let mut need_send_couunt = 0;
                'send_to_client: while need_send_couunt < MAX_SEND_COUNT {
                    let r = tx.send(Ok(true)).await;
                    match r {
                        Ok(_) => {
                            break 'send_to_client;
                        }
                        Err(e) => {
                            leptos::logging::error!("event_source error: {:?}", e);
                            if e.is_full() {
                                need_send_couunt = need_send_couunt + 1;
                                tokio::time::sleep(tokio::time::Duration::from_millis(MAX_SEND_TIMEOUT)).await;
                                continue;
                            }
                            if e.is_disconnected() {
                                break 'app_events;
                            }
                        }
                    }
                }
                if need_send_couunt >= MAX_SEND_COUNT {
                    leptos::logging::error!("basket_state_notify : {:?}", "need_send_couunt >= MAX_SEND_COUNT");
                    break 'app_events;
                }
            }
        }) as Pin<Box<dyn Future<Output = ()> + Send + 'static>>);

        futures.push(Box::pin(async move { while let Some(_event) = input.next().await {} }) as Pin<Box<dyn Future<Output = ()> + Send + 'static>>);
        // wait for any completed future
        if let Some(_result) = futures.next().await {}
        unsubscribe.unsubscribe().await;
    });

    Ok(rx.into())
}
