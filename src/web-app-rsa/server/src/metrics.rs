use std::sync::OnceLock;
use std::time::Instant;

use axum::extract::Request;
use axum::middleware::Next;
use axum::response::Response;
use opentelemetry::global;
use opentelemetry::metrics::{Counter, Histogram, UpDownCounter};
use opentelemetry::KeyValue;

struct HttpMetrics {
    request_count: Counter<u64>,
    request_duration: Histogram<f64>,
    active_requests: UpDownCounter<i64>,
}

fn metrics() -> &'static HttpMetrics {
    static METRICS: OnceLock<HttpMetrics> = OnceLock::new();
    METRICS.get_or_init(|| {
        let meter = global::meter("webapprsa3");
        HttpMetrics {
            request_count: meter.u64_counter("http.server.request_count").build(),
            request_duration: meter.f64_histogram("http.server.request_duration_seconds").build(),
            active_requests: meter.i64_up_down_counter("http.server.active_requests").build(),
        }
    })
}

pub async fn metrics_middleware(request: Request, next: Next) -> Response {
    let start = Instant::now();
    let method = request.method().to_string();
    let path = request.uri().path().to_string();

    let m = metrics();
    m.active_requests
        .add(1, &[KeyValue::new("method", method.clone())]);

    let span = tracing::info_span!(
        "http.request",
        method = %method,
        path = %path,
    );
    let _guard = span.enter();

    let response = next.run(request).await;

    drop(_guard);

    let status = response.status().as_u16();
    let duration = start.elapsed().as_secs_f64();

    m.request_count.add(
        1,
        &[
            KeyValue::new("method", method.clone()),
            KeyValue::new("path", path.clone()),
            KeyValue::new("status_code", status.to_string()),
        ],
    );
    m.request_duration.record(
        duration,
        &[
            KeyValue::new("method", method.clone()),
            KeyValue::new("path", path.clone()),
            KeyValue::new("status_code", status.to_string()),
        ],
    );
    m.active_requests
        .add(-1, &[KeyValue::new("method", method)]);

    response
}
