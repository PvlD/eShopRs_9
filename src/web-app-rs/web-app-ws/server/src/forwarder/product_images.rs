use axum::extract::{Path, Request};
use axum::http::StatusCode;

use axum::{
    extract::State,
    response::{IntoResponse, Response},
    routing::get,
};

use futures::Future;

use super::Client;
use crate::url_mapper::UrlMapService;
use log::error;

fn handler_get(Path(id): Path<usize>, State(state): State<LState>, req: Request) -> impl Future<Output = Response> {
    let q = if let Some(q) = req.uri().query() { format!("?{q}") } else { "".to_string() };

    let base_map_to_path = "http://catalog-api";
    let base = state.url_map_service.get_mapped_url(base_map_to_path).unwrap_or(base_map_to_path);

    //info!("Forwarding request to {base}");

    let ext = format!("/api/catalog/items/{}/pic", id);
    let uri = format!("{base}{ext}{q}");

    async move {
        super::forward(&uri, req, state.client).await.unwrap_or_else(|err| {
            error!("Error forwarding request: {}", err);
            (StatusCode::INTERNAL_SERVER_ERROR).into_response()
        })
    }
}

#[derive(Clone)]
struct LState {
    client: Client,
    url_map_service: UrlMapService,
}

pub fn router(url_map_service: UrlMapService) -> axum::Router {
    let state = LState { client: Client::new(), url_map_service };

    let app = axum::Router::new().route("/product-images/{id}", get(handler_get));
    app.with_state(state)
}
