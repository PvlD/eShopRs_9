use crate::error::AppError;
use leptos::prelude::*;
use serde::{Deserialize, Serialize};
use url::form_urlencoded::Serializer;

pub fn login_url(current_path: &str) -> String {
    let query = Serializer::new(String::new())
        .append_pair("return_url", current_path)
        .finish();
    format!("/user/login?{}", query)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthSession {
    pub is_authenticated: bool,
    pub user_name: Option<String>,
}

#[server(GetAuthSession, "/api/auth/session")]
pub async fn get_auth_session() -> Result<AuthSession, AppError> {
    use tower_sessions::Session;
    let session: Session = leptos_axum::extract().await.map_err(|e| AppError(e.to_string()))?;
    let is_authenticated = session.get("authenticated").await.map_err(|e| AppError(e.to_string()))?.unwrap_or(false);
    let user_name = session.get("user_name").await.map_err(|e| AppError(e.to_string()))?;
    Ok(AuthSession { is_authenticated, user_name })
}
