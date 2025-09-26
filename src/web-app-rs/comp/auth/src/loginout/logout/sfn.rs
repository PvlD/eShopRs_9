use super::super::super::*;

#[server(prefix = "/user")]
pub async fn logout() -> Result<(), crate::AppError> {
    leptos::logging::log!("fn logout() ");

    use leptos_axum::extract;
    use users::AuthSession;

    let mut auth_session: AuthSession = extract().await?;
    auth_session.logout().await.map_err(|e| crate::AppError::Other(e.to_string()))?;

    Ok(())
}
