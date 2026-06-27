use std::sync::Arc;

use axum::{
    extract::{Query, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Redirect},
};
use serde::Deserialize;
use tower_sessions::Session;

use settings::Settings;

const SCOPES: &str = "openid profile orders basket offline_access";

pub async fn login(
    headers: HeaderMap,
    State(settings): State<Arc<Settings>>,
    session: Session,
    Query(params): Query<LoginParams>,
) -> Result<impl IntoResponse, AppError> {
    let state = random_string();
    let nonce = random_string();

    session.insert("oauth_state", &state).await?;
    session.insert("oauth_nonce", &nonce).await?;
    if let Some(return_url) = params.return_url {
        session.insert("return_url", &return_url).await?;
    }

    let host = headers
        .get("host")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("localhost:3030");
    let port = host.split(':').nth(1).unwrap_or("3030");
    let scheme = if port == "8443" || port == "4433" { "https" } else { "http" };
    let base_url = format!("{scheme}://{host}");
    let redirect_uri = format!("{base_url}/signin-oidc");
    session.insert("oauth_redirect_uri", &redirect_uri).await?;

    let auth_url = format!(
        "{}/connect/authorize?client_id={}&response_type=code&scope={}&redirect_uri={}&state={}&nonce={}",
        settings.identity.authority,
        settings.identity.client_id,
        urlencoding(SCOPES),
        urlencoding(&redirect_uri),
        state,
        nonce,
    );

    Ok(Redirect::to(&auth_url))
}

#[derive(Deserialize)]
pub(crate) struct LoginParams {
    pub(crate) return_url: Option<String>,
}

#[derive(Deserialize)]
pub(crate) struct CallbackParams {
    pub(crate) code: String,
    pub(crate) state: String,
}

pub(crate) async fn callback(
    State(settings): State<Arc<Settings>>,
    session: Session,
    Query(params): Query<CallbackParams>,
) -> Result<impl IntoResponse, AppError> {
    let saved_state = session
        .get::<String>("oauth_state")
        .await?
        .ok_or(AppError("No state in session".into()))?;

    if saved_state != params.state {
        return Err(AppError("State mismatch".into()));
    }

    let redirect_uri = session
        .get::<String>("oauth_redirect_uri")
        .await?
        .unwrap_or_else(|| format!("{}/signin-oidc", settings.identity.callback_url));
    let http = reqwest::Client::new();

    let token_body = format!(
        "grant_type=authorization_code&code={}&redirect_uri={}&client_id={}&client_secret={}",
        urlencoding(&params.code),
        urlencoding(&redirect_uri),
        urlencoding(&settings.identity.client_id),
        urlencoding(&settings.identity.client_secret),
    );
    let token_resp = http
        .post(format!("{}/connect/token", settings.identity.authority))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(token_body)
        .send()
        .await
        .map_err(|e| AppError(format!("Token request failed: {e}")))?;

    if !token_resp.status().is_success() {
        let body = token_resp.text().await.unwrap_or_default();
        return Err(AppError(format!("Token exchange failed: {body}")));
    }

    let token_data: TokenResponse = token_resp
        .json()
        .await
        .map_err(|e| AppError(format!("Failed to parse token response: {e}")))?;

    let user_name = extract_username_from_id_token(&token_data.id_token)?;

    session.insert("authenticated", true).await?;
    session.insert("user_name", &user_name).await?;
    session.insert("access_token", &token_data.access_token).await?;
    session.insert("id_token", &token_data.id_token).await?;
    if let Some(ref rt) = token_data.refresh_token {
        session.insert("refresh_token", rt).await?;
    }

    let userinfo_resp = http
        .get(format!("{}/connect/userinfo", settings.identity.authority))
        .bearer_auth(&token_data.access_token)
        .send()
        .await
        .map_err(|e| AppError(format!("UserInfo request failed: {e}")))?;
    if userinfo_resp.status().is_success() {
        if let Ok(userinfo) = userinfo_resp.json::<serde_json::Value>().await {
            if let Some(val) = userinfo.get("sub").and_then(|v| v.as_str()) {
                let _ = session.insert("sub", val).await;
            }
            for key in ["address_street", "address_city", "address_state", "address_country", "address_zip_code"] {
                if let Some(val) = userinfo.get(key).and_then(|v| v.as_str()) {
                    let _ = session.insert(key, val).await;
                }
            }
        }
    }

    session.remove::<String>("oauth_state").await?;
    session.remove::<String>("oauth_nonce").await?;
    session.remove::<String>("oauth_redirect_uri").await?;
    let return_url = session
        .remove::<String>("return_url")
        .await?
        .unwrap_or_else(|| "/".to_string());

    Ok(Redirect::to(&return_url))
}

pub async fn logout(
    headers: HeaderMap,
    State(settings): State<Arc<Settings>>,
    session: Session,
) -> Result<impl IntoResponse, AppError> {
    let id_token: Option<String> = session.get("id_token").await?;
    session.clear().await;

    let host = headers
        .get("host")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("localhost:3030");
    let port = host.split(':').nth(1).unwrap_or("3030");
    let scheme = if port == "8443" || port == "4433" { "https" } else { "http" };
    let base_url = format!("{scheme}://{host}");
    let post_logout_uri = format!("{base_url}/signout-callback-oidc");
    let mut logout_url = format!(
        "{}/connect/endsession?post_logout_redirect_uri={}",
        settings.identity.authority,
        urlencoding(&post_logout_uri)
    );

    if let Some(ref token) = id_token {
        logout_url = format!("{}&id_token_hint={}", logout_url, token);
    }

    Ok(Redirect::to(&logout_url))
}

pub async fn logout_callback() -> impl IntoResponse {
    Redirect::to("/")
}

#[derive(Deserialize)]
struct TokenResponse {
    access_token: String,
    id_token: String,
    #[serde(default)]
    refresh_token: Option<String>,
}

fn extract_username_from_id_token(id_token: &str) -> Result<String, AppError> {
    let payload = decode_jwt_payload(id_token)?;
    let user_name = payload
        .get("preferred_username")
        .and_then(|v| v.as_str())
        .or_else(|| payload.get("name").and_then(|v| v.as_str()))
        .or_else(|| payload.get("email").and_then(|v| v.as_str()))
        .unwrap_or("User")
        .to_string();
    Ok(user_name)
}

fn decode_jwt_payload(token: &str) -> Result<serde_json::Value, AppError> {
    let parts: Vec<&str> = token.split('.').collect();
    let payload = parts.get(1).ok_or(AppError("Invalid JWT".into()))?;
    let decoded = base64_url_decode(payload)?;
    serde_json::from_slice(&decoded).map_err(|e| AppError(format!("JWT payload parse failed: {e}")))
}

fn base64_url_decode(input: &str) -> Result<Vec<u8>, AppError> {
    let standard = input.replace('-', "+").replace('_', "/");
    let padded = match standard.len() % 4 {
        0 => standard,
        n => {
            let mut s = standard;
            for _ in 0..(4 - n) {
                s.push('=');
            }
            s
        }
    };
    use base64::Engine;
    base64::engine::general_purpose::STANDARD
        .decode(padded.as_bytes())
        .map_err(|e| AppError(format!("Base64 decode failed: {e}")))
}

fn random_string() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let t = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    format!("{t:x}")
}

fn urlencoding(s: &str) -> String {
    let mut encoded = String::new();
    for byte in s.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                encoded.push(byte as char);
            }
            _ => {
                encoded.push_str(&format!("%{:02X}", byte));
            }
        }
    }
    encoded
}

pub struct AppError(pub String);

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        (StatusCode::INTERNAL_SERVER_ERROR, self.0).into_response()
    }
}

impl From<tower_sessions::session::Error> for AppError {
    fn from(e: tower_sessions::session::Error) -> Self {
        AppError(e.to_string())
    }
}
