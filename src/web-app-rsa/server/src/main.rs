#![recursion_limit = "256"]

use std::sync::Arc;

use axum::{
    extract::{FromRef, Path, State},
    http::{header, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use leptos::context::provide_context;
use leptos::logging::log;
use leptos::prelude::*;
use leptos_axum::{generate_route_list, LeptosRoutes};
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

#[tokio::main]
async fn main() {
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

    let app_state = AppState {
        leptos_options: leptos_options.clone(),
        settings: settings.clone(),
        notification_service: notification_service.clone(),
    };

    let ctx_provider = {
        let s = settings.clone();
        move || provide_context(s.clone())
    };

    let lo_routes = leptos_options.clone();
    let lo_fallback = leptos_options.clone();

    let session_store = MemoryStore::default();
    let session_layer = SessionManagerLayer::new(session_store)
        .with_secure(false)
        .with_same_site(SameSite::Lax)
        .with_name("webapprsa3.sess");

    let app = {
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
    .leptos_routes_with_context(
            &app_state,
            routes,
            ctx_provider.clone(),
            move || shell(lo_routes.clone()),
        )
        .layer(session_layer)
        .layer(axum::middleware::from_fn(metrics::metrics_middleware))
        .fallback(leptos_axum::file_and_error_handler_with_context::<AppState, _>(
            ctx_provider,
            move |_req| shell(lo_fallback.clone()),
        ))
        .with_state(app_state);

    log!("listening on http://{}", &addr);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
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
