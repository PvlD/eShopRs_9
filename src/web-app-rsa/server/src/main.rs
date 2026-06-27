#![recursion_limit = "256"]

use std::io::BufReader;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use anyhow::Context;
use axum::{
    extract::{ws::WebSocketUpgrade, FromRef, Path, Request, State},
    http::{header, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    routing::{get, post},
    Router,
};
use axum_server::tls_rustls::RustlsConfig;
use leptos::config::ReloadWSProtocol;
use leptos::context::provide_context;
use leptos::prelude::*;
use leptos_axum::{generate_route_list, LeptosRoutes};
use rustls::pki_types::CertificateDer;
use tokio::sync::watch;
use tower_sessions::{cookie::SameSite, MemoryStore, SessionManagerLayer};

use app::*;
use eventbus::EventBusBuilder;
use settings::Settings;

mod auth;
mod events;
mod handlers;
mod metrics;
mod notification;

#[derive(Clone)]
struct AppState {
    leptos_options: LeptosOptions,
    settings: Arc<Settings>,
    notification_service: Arc<notification::OrderStatusNotificationService>,
}

impl FromRef<AppState> for LeptosOptions {
    fn from_ref(state: &AppState) -> Self {
        state.leptos_options.clone()
    }
}

impl FromRef<AppState> for Arc<Settings> {
    fn from_ref(state: &AppState) -> Self {
        state.settings.clone()
    }
}

impl FromRef<AppState> for Arc<notification::OrderStatusNotificationService> {
    fn from_ref(state: &AppState) -> Self {
        state.notification_service.clone()
    }
}

struct TelemetryProviders {
    _tracer_provider: Option<opentelemetry_sdk::trace::SdkTracerProvider>,
    _meter_provider: Option<opentelemetry_sdk::metrics::SdkMeterProvider>,
}

fn init_telemetry(otel: &settings::OtelSettings) -> TelemetryProviders {
    use opentelemetry::trace::TracerProvider as _;
    use opentelemetry_otlp::WithExportConfig;
    use tracing_subscriber::prelude::*;

    let endpoint = otel.exporter_otlp_endpoint.clone();
    let service_name = otel.service_name.clone();
    let resource_attributes = otel.resource_attributes.clone();
    let traces_sampler = otel.traces_sampler.clone();
    let bsp_schedule_delay = otel.bsp_schedule_delay;
    let metric_export_interval = otel.metric_export_interval;

    let (tracer_provider, meter_provider) = if let Some(endpoint) = endpoint {
        let span_exporter = opentelemetry_otlp::SpanExporter::builder()
            .with_tonic()
            .with_endpoint(&endpoint)
            .build()
            .expect("Failed to build OTLP span exporter");

        let span_processor = if let Some(delay) = bsp_schedule_delay {
            let config = opentelemetry_sdk::trace::BatchConfigBuilder::default()
                .with_scheduled_delay(std::time::Duration::from_millis(delay))
                .build();
            opentelemetry_sdk::trace::BatchSpanProcessor::builder(span_exporter)
                .with_batch_config(config)
                .build()
        } else {
            opentelemetry_sdk::trace::BatchSpanProcessor::builder(span_exporter).build()
        };

        let mut resource_builder = opentelemetry_sdk::Resource::builder();
        if let Some(ref name) = service_name {
            resource_builder = resource_builder.with_service_name(name.clone());
        }
        if let Some(ref attrs) = resource_attributes {
            let kvs: Vec<opentelemetry::KeyValue> = attrs.split(',')
                .filter_map(|pair| {
                    let eq = pair.find('=')?;
                    Some((pair[..eq].trim().to_string(), pair[eq + 1..].trim().to_string()))
                })
                .map(|(k, v)| opentelemetry::KeyValue::new(k, v))
                .collect();
            if !kvs.is_empty() {
                resource_builder = resource_builder.with_attributes(kvs);
            }
        }
        let resource = resource_builder.build();

        let mut tracer_provider_builder = opentelemetry_sdk::trace::SdkTracerProvider::builder()
            .with_span_processor(span_processor)
            .with_resource(resource.clone());

        if let Some(sampler) = traces_sampler {
            let s = match sampler.to_lowercase().as_str() {
                "always_on" => opentelemetry_sdk::trace::Sampler::AlwaysOn,
                "always_off" => opentelemetry_sdk::trace::Sampler::AlwaysOff,
                _ => {
                    tracing::warn!("unknown sampler '{sampler}', falling back to AlwaysOn");
                    opentelemetry_sdk::trace::Sampler::AlwaysOn
                }
            };
            tracer_provider_builder = tracer_provider_builder.with_sampler(s);
        }

        let tracer_provider = tracer_provider_builder.build();

        let metric_exporter = opentelemetry_otlp::MetricExporter::builder()
            .with_tonic()
            .with_endpoint(&endpoint)
            .build()
            .expect("Failed to build OTLP metric exporter");

        let reader = match metric_export_interval {
            Some(interval) => opentelemetry_sdk::metrics::PeriodicReader::builder(metric_exporter)
                .with_interval(std::time::Duration::from_millis(interval))
                .build(),
            None => opentelemetry_sdk::metrics::PeriodicReader::builder(metric_exporter).build(),
        };

        let meter_provider = opentelemetry_sdk::metrics::SdkMeterProvider::builder()
            .with_reader(reader)
            .with_resource(resource)
            .build();

        opentelemetry::global::set_meter_provider(meter_provider.clone());

        (Some(tracer_provider), Some(meter_provider))
    } else {
        (None, None)
    };

    let env_filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"));

    let fmt_layer = tracing_subscriber::fmt::layer().with_target(true);

    let subscriber = tracing_subscriber::registry()
        .with(fmt_layer)
        .with(env_filter);

    if let Some(ref provider) = tracer_provider {
        let otel_layer =
            tracing_opentelemetry::OpenTelemetryLayer::new(provider.tracer("webapprsa3"));
        subscriber.with(otel_layer).init();
    } else {
        subscriber.init();
    }

    TelemetryProviders {
        _tracer_provider: tracer_provider,
        _meter_provider: meter_provider,
    }
}

async fn alt_svc(req: Request, next: Next) -> Response {
    let mut res = next.run(req).await;
    res.headers_mut().insert(
        header::ALT_SVC,
        header::HeaderValue::from_static("h3=\":4433\""),
    );
    res
}

async fn handle_live_reload(ws: WebSocketUpgrade) -> impl axum::response::IntoResponse {
    ws.on_upgrade(move |mut browser| async move {
        while let Some(Ok(_)) = browser.recv().await {}
    })
}

fn build_routes() -> Router<AppState> {
    let mut router = Router::<AppState>::new()
        .route("/product-images/{id}", get(proxy_product_image))
        .route("/user/login", get(auth::login))
        .route("/signin-oidc", get(auth::callback))
        .route("/user/logout", post(auth::logout))
        .route("/signout-callback-oidc", get(auth::logout_callback))
        .route("/api/orders/events", get(notification::order_events_handler));

    #[cfg(debug_assertions)]
    {
        router = router
            .route("/health", get(|| async { StatusCode::OK }))
            .route("/alive", get(|| async { StatusCode::OK }));
    }

    router
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    rustls::crypto::ring::default_provider()
        .install_default()
        .expect("failed to install rustls crypto provider");

    dotenvy::dotenv().ok();

    let settings = Arc::new(settings::load().expect("Failed to load config"));
    let _telemetry = init_telemetry(&settings.otel);

    let notification_service =
        Arc::new(notification::OrderStatusNotificationService::new());
    let notification_handler =
        handlers::OrderStatusNotificationHandler::new(notification_service.clone());

    let (_event_bus, _consumer_handle) = EventBusBuilder::new()
        .url(&settings.eventbus.url)
        .subscription_client_name(&settings.eventbus.subscription_client_name)
        .retry_count(settings.eventbus.retry_count)
        .add_subscription::<events::OrderStatusChangedToSubmittedIntegrationEvent, handlers::OrderStatusChangedHandler>(
            handlers::OrderStatusChangedHandler,
        )
        .add_subscription::<events::OrderStatusChangedToSubmittedIntegrationEvent, handlers::OrderStatusNotificationHandler>(
            notification_handler.clone(),
        )
        .add_subscription::<events::OrderStatusChangedToAwaitingValidationIntegrationEvent, handlers::OrderStatusChangedHandler>(
            handlers::OrderStatusChangedHandler,
        )
        .add_subscription::<events::OrderStatusChangedToAwaitingValidationIntegrationEvent, handlers::OrderStatusNotificationHandler>(
            notification_handler.clone(),
        )
        .add_subscription::<events::OrderStatusChangedToStockConfirmedIntegrationEvent, handlers::OrderStatusChangedHandler>(
            handlers::OrderStatusChangedHandler,
        )
        .add_subscription::<events::OrderStatusChangedToStockConfirmedIntegrationEvent, handlers::OrderStatusNotificationHandler>(
            notification_handler.clone(),
        )
        .add_subscription::<events::OrderStatusChangedToPaidIntegrationEvent, handlers::OrderStatusChangedHandler>(
            handlers::OrderStatusChangedHandler,
        )
        .add_subscription::<events::OrderStatusChangedToPaidIntegrationEvent, handlers::OrderStatusNotificationHandler>(
            notification_handler.clone(),
        )
        .add_subscription::<events::OrderStatusChangedToShippedIntegrationEvent, handlers::OrderStatusChangedHandler>(
            handlers::OrderStatusChangedHandler,
        )
        .add_subscription::<events::OrderStatusChangedToShippedIntegrationEvent, handlers::OrderStatusNotificationHandler>(
            notification_handler.clone(),
        )
        .add_subscription::<events::OrderStatusChangedToCancelledIntegrationEvent, handlers::OrderStatusChangedHandler>(
            handlers::OrderStatusChangedHandler,
        )
        .add_subscription::<events::OrderStatusChangedToCancelledIntegrationEvent, handlers::OrderStatusNotificationHandler>(
            notification_handler.clone(),
        )
        .build()
        .await
        .expect("Failed to build event bus");

    let conf = get_configuration(None).unwrap();
    let leptos_options = conf.leptos_options;
    let addr = leptos_options.site_addr;
    let routes = generate_route_list(App);

    // TLS options for HTTPS and QUIC servers
    let mut tls_options = leptos_options.clone();
    tls_options.reload_external_port = Some(3031);
    tls_options.reload_ws_protocol = ReloadWSProtocol::WS;

    // App states
    let app_state = AppState {
        leptos_options: leptos_options.clone(),
        settings: settings.clone(),
        notification_service: notification_service.clone(),
    };
    let app_state_tls = AppState {
        leptos_options: tls_options.clone(),
        settings: settings.clone(),
        notification_service: notification_service.clone(),
    };
    let ctx_provider = {
        let s = settings.clone();
        move || provide_context(s.clone())
    };

    let store = MemoryStore::default();
    let session_layer_plain = SessionManagerLayer::new(store.clone())
        .with_secure(false)
        .with_same_site(SameSite::Lax)
        .with_name("webapprsa3.sess");
    let session_layer_tls = SessionManagerLayer::new(store)
        .with_secure(false)
        .with_same_site(SameSite::Lax)
        .with_name("webapprsa3.sess");

    // ── Plain HTTP router (port 3030) ──
    let lo_routes = leptos_options.clone();
    let lo_fallback = leptos_options.clone();

    let app_http = build_routes()
        .leptos_routes_with_context(
            &app_state,
            routes.clone(),
            ctx_provider.clone(),
            move || shell(lo_routes.clone()),
        )
        .layer(session_layer_plain)
        .layer(axum::middleware::from_fn(metrics::metrics_middleware))
        .fallback(leptos_axum::file_and_error_handler_with_context::<AppState, _>(
            ctx_provider.clone(),
            move |_req| shell(lo_fallback.clone()),
        ))
        .with_state(app_state);

    // ── TLS/QUIC base router (without /live_reload) ──
    let tls_routes_lo = tls_options.clone();
    let tls_fallback_lo = tls_options.clone();

    let app_tls_base = build_routes()
        .leptos_routes_with_context(
            &app_state_tls,
            routes.clone(),
            ctx_provider.clone(),
            move || shell(tls_routes_lo.clone()),
        )
        .layer(session_layer_tls)
        .layer(axum::middleware::from_fn(metrics::metrics_middleware))
        .fallback(leptos_axum::file_and_error_handler_with_context::<AppState, _>(
            ctx_provider,
            move |_req| shell(tls_fallback_lo.clone()),
        ))
        .with_state(app_state_tls);

    let app_tls = app_tls_base
        .clone()
        .route("/live_reload", axum::routing::any(handle_live_reload))
        .layer(axum::middleware::from_fn(alt_svc));

    let app_for_quic = app_tls_base;

    let (shutdown_tx, mut shutdown_rx) = watch::channel(());

    tokio::spawn(async move {
        tokio::signal::ctrl_c().await.ok();
        tracing::info!("shutdown signal received");
        let _ = shutdown_tx.send(());
    });

    // ── Plain HTTP/1.1 ──
    let mut plain_sd = shutdown_rx.clone();
    let plain_task = tokio::spawn(async move {
        let listener = tokio::net::TcpListener::bind(&addr).await?;
        tracing::info!("HTTP/1.1 listening on http://{addr}");
        axum::serve(listener, app_http.into_make_service())
            .with_graceful_shutdown(async move {
                plain_sd.changed().await.ok();
            })
            .await
            .context("plain HTTP server exited with error")
    });

    // ── TCP+TLS: HTTP/1.1 (port 8443) ──
    let tcp_tls_addr: std::net::SocketAddr = ([127, 0, 0, 1], 8443).into();
    let cert_path = PathBuf::from("config/tls/cert.pem");
    let key_path = PathBuf::from("config/tls/key.pem");

    let mut reader = BufReader::new(
        std::fs::File::open(&cert_path)
            .with_context(|| format!("failed to open {}", cert_path.display()))?,
    );
    let certs: Vec<CertificateDer<'static>> = rustls_pemfile::certs(&mut reader)
        .collect::<Result<Vec<_>, _>>()
        .context("failed to parse certificate")?;

    let mut reader = BufReader::new(
        std::fs::File::open(&key_path)
            .with_context(|| format!("failed to open {}", key_path.display()))?,
    );
    let key = rustls_pemfile::private_key(&mut reader)
        .context("failed to parse private key")?
        .ok_or_else(|| anyhow::anyhow!("no private key found in {}", key_path.display()))?;

    let mut server_config = rustls::ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(certs, key)
        .context("failed to build TLS config")?;
    server_config.alpn_protocols = vec![b"h2".to_vec(), b"http/1.1".to_vec()];
    let tls_config = RustlsConfig::from_config(Arc::new(server_config));

    let tls_handle = axum_server::Handle::new();
    let mut tls_sd = shutdown_rx.clone();
    let tls_handle_clone = tls_handle.clone();
    tokio::spawn(async move {
        tls_sd.changed().await.ok();
        tls_handle_clone.graceful_shutdown(Some(Duration::from_secs(30)));
    });

    let tls_task = tokio::spawn({
        let app = app_tls;
        async move {
            tracing::info!("HTTP/1.1 (TLS) listening on https://{tcp_tls_addr}");
            axum_server::bind_rustls(tcp_tls_addr, tls_config)
                .handle(tls_handle)
                .serve(app.into_make_service())
                .await
                .context("TCP/TLS server exited with error")
        }
    });

    // ── QUIC: HTTP/3 (port 4433) ──
    let quic_addr: std::net::SocketAddr = ([127, 0, 0, 1], 4433).into();
    let quic_sd = shutdown_rx.clone();

    let mut reader = BufReader::new(
        std::fs::File::open(&cert_path)
            .with_context(|| format!("failed to open {}", cert_path.display()))?,
    );
    let certs: Vec<CertificateDer<'static>> = rustls_pemfile::certs(&mut reader)
        .collect::<Result<Vec<_>, _>>()
        .context("failed to parse certificate")?;

    let mut reader = BufReader::new(
        std::fs::File::open(&key_path)
            .with_context(|| format!("failed to open {}", key_path.display()))?,
    );
    let key = rustls_pemfile::private_key(&mut reader)
        .context("failed to parse private key")?
        .ok_or_else(|| anyhow::anyhow!("no private key found in {}", key_path.display()))?;

    let mut quic_tls_config = rustls::ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(certs, key)
        .context("failed to build rustls TLS config")?;

    quic_tls_config.alpn_protocols = vec![b"h3".to_vec()];

    let quic_server_config = quinn::ServerConfig::with_crypto(Arc::new(
        quinn::crypto::rustls::QuicServerConfig::try_from(quic_tls_config)
            .context("failed to build QUIC TLS config")?,
    ));

    let quic_task = tokio::spawn(async move {
        let endpoint = quinn::Endpoint::server(quic_server_config, quic_addr)
            .context("failed to bind QUIC endpoint")?;
        tracing::info!("HTTP/3 listening on https://{quic_addr}");
        let mut shutdown = quic_sd;
        loop {
            tokio::select! {
                incoming = endpoint.accept() => {
                    match incoming {
                        Some(incoming) => {
                            let app = app_for_quic.clone();
                            tokio::spawn(async move {
                                match incoming.await {
                                    Ok(conn) => {
                                        let quic_conn = h3_quinn::Connection::new(conn);
                                        match h3::server::builder().build(quic_conn).await {
                                            Ok(mut h3_conn) => loop {
                                                match h3_conn.accept().await {
                                                    Ok(Some(resolver)) => {
                                                        if let Err(e) = h3_axum::serve_h3_with_axum(app.clone(), resolver).await {
                                                            tracing::error!("request error: {e}");
                                                        }
                                                    }
                                                    Ok(None) => break,
                                                    Err(e) if h3_axum::is_graceful_h3_close(&e) => {
                                                        tracing::debug!("h3 connection closed gracefully");
                                                        break;
                                                    }
                                                    Err(e) => {
                                                        tracing::error!("h3 connection error: {e}");
                                                        break;
                                                    }
                                                }
                                            },
                                            Err(e) => {
                                                tracing::error!("failed to build h3 connection: {e}");
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        tracing::error!("failed to accept QUIC connection: {e}");
                                    }
                                }
                            });
                        }
                        None => break,
                    }
                }
                _ = shutdown.changed() => {
                    tracing::info!("shutting down HTTP/3 server");
                    endpoint.close(0u8.into(), b"shutdown");
                    endpoint.wait_idle().await;
                    break;
                }
            }
        }
        anyhow::Ok::<()>(())
    });

    shutdown_rx.changed().await.ok();
    tracing::info!("shutting down");

    plain_task.await.unwrap()?;
    tls_task.await.unwrap()?;
    quic_task.await.unwrap()?;

    Ok(())
}

async fn proxy_product_image(
    Path(id): Path<i32>,
    State(settings): State<Arc<Settings>>,
) -> impl IntoResponse {
    let catalog_url = settings.services.catalog_api.http.as_str();
    let url = format!("{}/api/catalog/items/{}/pic?api-version=2.0", catalog_url, id);
    let client = reqwest::Client::new();
    let resp = match client.get(&url).send().await {
        Ok(r) => r,
        Err(e) => {
            return (StatusCode::BAD_GATEWAY, format!("upstream error: {}", e)).into_response();
        }
    };
    let status = resp.status();
    if !status.is_success() {
        return (StatusCode::from_u16(status.as_u16()).unwrap_or(StatusCode::BAD_GATEWAY), "upstream error".to_string()).into_response();
    }
    let content_type = resp
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("image/webp")
        .to_string();
    let bytes = match resp.bytes().await {
        Ok(b) => b,
        Err(e) => {
            return (StatusCode::BAD_GATEWAY, format!("failed to read body: {}", e)).into_response();
        }
    };
    ([(header::CONTENT_TYPE, content_type)], bytes).into_response()
}
