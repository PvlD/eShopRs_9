use std::collections::HashMap;
use std::sync::Arc;

use axum::extract::State;
use axum::response::sse::{Event, KeepAlive, Sse};
use axum::response::IntoResponse;
use futures::stream;
use std::convert::Infallible;
use tokio::sync::{Notify, RwLock};
use tower_sessions::Session;

pub struct OrderStatusNotificationService {
    notifiers: RwLock<HashMap<String, Arc<Notify>>>,
}

impl OrderStatusNotificationService {
    pub fn new() -> Self {
        Self {
            notifiers: RwLock::new(HashMap::new()),
        }
    }

    pub async fn subscribe(&self, buyer_id: &str) -> Arc<Notify> {
        let mut map = self.notifiers.write().await;
        map.entry(buyer_id.to_string())
            .or_insert_with(|| Arc::new(Notify::new()))
            .clone()
    }

    pub async fn notify(&self, buyer_id: &str) {
        let map = self.notifiers.read().await;
        if let Some(notify) = map.get(buyer_id) {
            notify.notify_waiters();
        }
    }
}

pub async fn order_events_handler(
    State(service): State<Arc<OrderStatusNotificationService>>,
    session: Session,
) -> impl IntoResponse {
    let buyer_id: String = session
        .get("sub")
        .await
        .ok()
        .flatten()
        .unwrap_or_default();

    if buyer_id.is_empty() {
        return (
            axum::http::StatusCode::UNAUTHORIZED,
            "Unauthorized",
        )
            .into_response();
    }

    let notify = service.subscribe(&buyer_id).await;

    let stream = stream::unfold(notify, |notify| async {
        notify.notified().await;
        Some((Ok::<_, Infallible>(Event::default().data("refresh")), notify))
    });

    Sse::new(stream)
        .keep_alive(KeepAlive::default())
        .into_response()
}
