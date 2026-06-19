use std::collections::HashMap;
use std::collections::hash_map::Entry;

use serde::Deserialize;
use config::{Config, File, FileFormat, ConfigError, builder::DefaultState};

#[derive(Debug, Clone, Deserialize)]
pub struct Settings {
    pub services: ServicesConfig,
    pub identity: IdentitySettings,
    pub eventbus: EventBusSettings,
    pub ai: Option<AIOptions>,
    #[serde(default)]
    pub otel: OtelSettings,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct OtelSettings {
    pub exporter_otlp_endpoint: Option<String>,
    pub exporter_otlp_headers: Option<String>,
    pub service_name: Option<String>,
    pub resource_attributes: Option<String>,
    pub traces_sampler: Option<String>,
    pub bsp_schedule_delay: Option<u64>,
    pub metric_export_interval: Option<u64>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AIOptions {
    pub openai: OpenAIOptions,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OpenAIOptions {
    pub chat_model: String,
    pub api_key: String,
    #[serde(default = "default_provider_url")]
    pub provider_url: String,
}

fn default_provider_url() -> String {
    "http://localhost:1234/v1".into()
}

#[derive(Debug, Clone, Deserialize)]
pub struct EventBusSettings {
    pub url: String,
    pub subscription_client_name: String,
    #[serde(default = "default_retry_count")]
    pub retry_count: u32,
}

fn default_retry_count() -> u32 {
    10
}

#[derive(Debug, Clone, Deserialize)]
pub struct ServicesConfig {
    #[serde(rename = "catalog-api")]
    pub catalog_api: ServiceEndpoint,
    #[serde(rename = "basket-api")]
    pub basket_api: ServiceEndpoint,
    #[serde(rename = "ordering-api")]
    pub ordering_api: ServiceEndpoint,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ServiceEndpoint {
    pub http: String,
    pub version: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct IdentitySettings {
    pub authority: String,
    pub client_id: String,
    pub client_secret: String,
    pub callback_url: String,
}

pub fn load() -> Result<Settings, ConfigError> {
    let builder = apply_env_overrides(
        Config::builder().add_source(File::with_name("config")),
    )?;
    validate(builder.build()?.try_deserialize()?)
}

pub fn load_from_str(toml: &str) -> Result<Settings, ConfigError> {
    let builder = apply_env_overrides(
        Config::builder().add_source(File::from_str(toml, FileFormat::Toml)),
    )?;
    validate(builder.build()?.try_deserialize()?)
}

fn apply_env_overrides(
    mut builder: config::ConfigBuilder<DefaultState>,
) -> Result<config::ConfigBuilder<DefaultState>, ConfigError> {
    let mut service_endpoints: HashMap<(String, String), (usize, String)> = HashMap::new();

    for (key, value) in std::env::vars() {
        let lower = key.to_lowercase();

        if key.eq_ignore_ascii_case("CallBackUrl") {
            builder = builder.set_override("identity.callback_url", value)?;
            continue;
        }

        if key.eq_ignore_ascii_case("ConnectionStrings__eventbus") {
            builder = builder.set_override("eventbus.url", value)?;
            continue;
        }

        let upper = key.to_uppercase();
        match upper.as_str() {
            "OTEL_EXPORTER_OTLP_ENDPOINT" => {
                builder = builder.set_override("otel.exporter_otlp_endpoint", value)?;
                continue;
            }
            "OTEL_EXPORTER_OTLP_HEADERS" => {
                builder = builder.set_override("otel.exporter_otlp_headers", value)?;
                continue;
            }
            "OTEL_SERVICE_NAME" => {
                builder = builder.set_override("otel.service_name", value)?;
                continue;
            }
            "OTEL_RESOURCE_ATTRIBUTES" => {
                builder = builder.set_override("otel.resource_attributes", value)?;
                continue;
            }
            "OTEL_TRACES_SAMPLER" => {
                builder = builder.set_override("otel.traces_sampler", value)?;
                continue;
            }
            "OTEL_BSP_SCHEDULE_DELAY" => {
                builder = builder.set_override("otel.bsp_schedule_delay", value)?;
                continue;
            }
            "OTEL_METRIC_EXPORT_INTERVAL" => {
                builder = builder.set_override("otel.metric_export_interval", value)?;
                continue;
            }
            _ => {}
        }

        if let Some(rest) = lower.strip_prefix("services__") {
            if rest.split("__").any(|p| p == "version") {
                continue;
            }
            let parts: Vec<&str> = rest.split("__").collect();
            if lower.starts_with("ai__") {
                continue;
            }
            if parts.len() == 3 {
                if let Ok(idx) = parts[2].parse::<usize>() {
                    let name = parts[0].to_string();
                    let label = parts[1].to_string();
                    match service_endpoints.entry((name.clone(), label.clone())) {
                        Entry::Occupied(mut o) => {
                            let (old_idx, _) = o.get().clone();
                            if idx < old_idx {
                                let old_url = o.insert((idx, value.clone())).1;
                                tracing::warn!(
                                    "services__{}__{}__{} ({}) replaced by lower index {}",
                                    name, label, old_idx, old_url, idx,
                                );
                            } else {
                                tracing::warn!(
                                    "services__{}__{}__{} ignored (index {} already accepted)",
                                    name, label, idx, old_idx,
                                );
                            }
                        }
                        Entry::Vacant(v) => {
                            v.insert((idx, value.clone()));
                        }
                    }
                    continue;
                }
            }
        }

        if lower.contains("__") {
            builder = builder.set_override(lower.replace("__", "."), value)?;
        }
    }

    for ((name, label), (_idx, url)) in &service_endpoints {
        builder = builder.set_override(format!("services.{name}.{label}"), url.as_str())?;
    }

    Ok(builder)
}

fn validate(settings: Settings) -> Result<Settings, ConfigError> {
    if settings.services.catalog_api.version.is_none() {
        return Err(ConfigError::Message(
            "services.catalog-api.version is required in config.toml".into(),
        ));
    }
    if settings.services.ordering_api.version.is_none() {
        return Err(ConfigError::Message(
            "services.ordering-api.version is required in config.toml".into(),
        ));
    }
    Ok(settings)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    static ENV_LOCK: Mutex<()> = Mutex::new(());

    fn with_env<T>(vars: Vec<(&str, &str)>, f: impl FnOnce() -> T) -> T {
        let _guard = ENV_LOCK.lock().unwrap();
        let saved: Vec<(String, Option<String>)> = vars.iter()
            .map(|(k, _)| (k.to_string(), std::env::var(k).ok()))
            .collect();
        for (k, v) in &vars {
            std::env::set_var(k, v);
        }
        let result = f();
        for (k, old) in saved {
            match old {
                Some(v) => std::env::set_var(&k, v),
                None => std::env::remove_var(&k),
            }
        }
        result
    }

    fn toml_valid() -> &'static str {
        r#"
[services.catalog-api]
http = "http://catalog:5222"
version = "2.0"

[services.basket-api]
http = "http://basket:5221"

[services.ordering-api]
http = "http://ordering:5224"
version = "1.0"

[identity]
authority = "http://identity:5223"
client_id = "test-client"
client_secret = "test-secret"
callback_url = "http://localhost:3000"

[eventbus]
url = "amqp://guest:password@localhost:57589"
subscription_client_name = "WebApp"
retry_count = 10
"#
    }

    #[test]
    fn load_valid_config() {
        let _guard = ENV_LOCK.lock().unwrap();
        let s = load_from_str(toml_valid()).unwrap();
        assert_eq!(s.services.catalog_api.http, "http://catalog:5222");
        assert_eq!(s.services.catalog_api.version.as_deref(), Some("2.0"));
        assert_eq!(s.services.basket_api.http, "http://basket:5221");
        assert_eq!(s.services.basket_api.version, None);
        assert_eq!(s.services.ordering_api.http, "http://ordering:5224");
        assert_eq!(s.services.ordering_api.version.as_deref(), Some("1.0"));
        assert_eq!(s.identity.authority, "http://identity:5223");
        assert_eq!(s.identity.client_id, "test-client");
        assert_eq!(s.identity.client_secret, "test-secret");
        assert_eq!(s.identity.callback_url, "http://localhost:3000");
    }

    #[test]
    fn missing_catalog_api_version() {
        let _guard = ENV_LOCK.lock().unwrap();
        let toml = r#"
[services.catalog-api]
http = "http://catalog:5222"

[services.basket-api]
http = "http://basket:5221"

[services.ordering-api]
http = "http://ordering:5224"
version = "1.0"

[identity]
authority = "http://identity:5223"
client_id = "test-client"
client_secret = "test-secret"
callback_url = "http://localhost:3000"

[eventbus]
url = "amqp://guest:password@localhost:57589"
subscription_client_name = "WebApp"
retry_count = 10
"#;
        let err = load_from_str(toml).unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("catalog-api"), "error should mention catalog-api, got: {msg}");
    }

    #[test]
    fn missing_ordering_api_version() {
        let _guard = ENV_LOCK.lock().unwrap();
        let toml = r#"
[services.catalog-api]
http = "http://catalog:5222"
version = "2.0"

[services.basket-api]
http = "http://basket:5221"

[services.ordering-api]
http = "http://ordering:5224"

[identity]
authority = "http://identity:5223"
client_id = "test-client"
client_secret = "test-secret"
callback_url = "http://localhost:3000"

[eventbus]
url = "amqp://guest:password@localhost:57589"
subscription_client_name = "WebApp"
retry_count = 10
"#;
        let err = load_from_str(toml).unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("ordering-api"), "error should mention ordering-api, got: {msg}");
    }

    #[test]
    fn env_var_overrides_service_http() {
        let s = with_env(
            vec![("services__catalog-api__http__0", "http://override:9999")],
            || load_from_str(toml_valid()).unwrap(),
        );
        assert_eq!(s.services.catalog_api.http, "http://override:9999");
    }

    #[test]
    fn env_var_lower_index_wins() {
        let s = with_env(
            vec![
                ("services__catalog-api__http__1", "http://first:1111"),
                ("services__catalog-api__http__0", "http://second:2222"),
            ],
            || load_from_str(toml_valid()).unwrap(),
        );
        assert_eq!(s.services.catalog_api.http, "http://second:2222");
        assert_eq!(s.services.catalog_api.version.as_deref(), Some("2.0"));
    }

    #[test]
    fn env_var_general_override() {
        let s = with_env(
            vec![("identity__authority", "http://override-id:1111")],
            || load_from_str(toml_valid()).unwrap(),
        );
        assert_eq!(s.identity.authority, "http://override-id:1111");
    }

    #[test]
    fn env_var_callback_url() {
        let s = with_env(
            vec![("CallBackUrl", "http://override-cb:9999")],
            || load_from_str(toml_valid()).unwrap(),
        );
        assert_eq!(s.identity.callback_url, "http://override-cb:9999");
    }

    #[test]
    fn env_var_version_skipped() {
        let s = with_env(
            vec![
                ("services__catalog-api__version__0", "9.9"),
                ("services__ordering-api__version__1", "8.8"),
            ],
            || load_from_str(toml_valid()).unwrap(),
        );
        assert_eq!(s.services.catalog_api.version.as_deref(), Some("2.0"));
        assert_eq!(s.services.ordering_api.version.as_deref(), Some("1.0"));
    }

    #[test]
    fn env_var_version_keypath_skipped() {
        let s = with_env(
            vec![("services__catalog-api__version", "9.9")],
            || load_from_str(toml_valid()).unwrap(),
        );
        assert_eq!(s.services.catalog_api.version.as_deref(), Some("2.0"));
    }

    #[test]
    fn otel_env_var_overrides_endpoint() {
        let s = with_env(
            vec![("OTEL_EXPORTER_OTLP_ENDPOINT", "http://otel-test:4317")],
            || load_from_str(toml_valid()).unwrap(),
        );
        assert_eq!(
            s.otel.exporter_otlp_endpoint.as_deref(),
            Some("http://otel-test:4317")
        );
    }

    #[test]
    fn otel_env_var_overrides_service_name() {
        let s = with_env(
            vec![("OTEL_SERVICE_NAME", "test-webapp")],
            || load_from_str(toml_valid()).unwrap(),
        );
        assert_eq!(s.otel.service_name.as_deref(), Some("test-webapp"));
    }

    #[test]
    fn otel_section_from_toml() {
        let toml = r#"
[services.catalog-api]
http = "http://catalog:5222"
version = "2.0"

[services.basket-api]
http = "http://basket:5221"

[services.ordering-api]
http = "http://ordering:5224"
version = "1.0"

[identity]
authority = "http://identity:5223"
client_id = "test-client"
client_secret = "test-secret"
callback_url = "http://localhost:3000"

[eventbus]
url = "amqp://guest:password@localhost:57589"
subscription_client_name = "WebApp"
retry_count = 10

[otel]
exporter_otlp_endpoint = "http://config:4317"
service_name = "config-webapp"
bsp_schedule_delay = 2000
"#;
        let s = load_from_str(toml).unwrap();
        assert_eq!(
            s.otel.exporter_otlp_endpoint.as_deref(),
            Some("http://config:4317")
        );
        assert_eq!(s.otel.service_name.as_deref(), Some("config-webapp"));
        assert_eq!(s.otel.bsp_schedule_delay, Some(2000));
    }

    #[test]
    fn otel_defaults_when_missing() {
        let s = load_from_str(toml_valid()).unwrap();
        assert_eq!(s.otel.exporter_otlp_endpoint, None);
        assert_eq!(s.otel.service_name, None);
        assert_eq!(s.otel.bsp_schedule_delay, None);
    }
}
