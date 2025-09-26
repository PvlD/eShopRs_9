use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};
pub fn trace_init() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                //.unwrap_or_else(|_|  format!("{}=trace", env!("CARGO_PKG_NAME") ).into() ),
                .unwrap_or_else(|_| EnvFilter::new("trace")),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
}
