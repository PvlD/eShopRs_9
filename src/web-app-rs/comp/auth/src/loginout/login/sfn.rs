use super::super::super::*;

#[server(prefix = "/user")]
//#[middleware(crate::middleware::LoggingLayer)]
pub async fn login(next_url: String) -> Result<(), crate::AppError> {
    use axum_login::tower_sessions::Session;
    use leptos_axum::extract;
    use users::AuthSession;

    let auth_session: AuthSession = extract().await?;
    let session: Session = extract().await?;

    //println!("login start  next_url: {}", next_url);

    //tokio::time::sleep(std::time::Duration::from_millis(2500)).await;

    let (auth_url, csrf_state, nonce) = auth_session.backend.authorize_url_with_scopes(vec!["openid", "profile", "orders", "basket"]);

    session.insert(CSRF_STATE_KEY, csrf_state.secret()).await.expect("csrf_state Serialization should not fail.");

    session.insert(NONCE_STATE_KEY, nonce.secret()).await.expect("nonce Serialization should not fail.");

    session.insert(NEXT_URL_KEY, next_url).await.unwrap_or_else(|_| panic!("{} Serialization should not fail.", stringify!(next_url)));

    println!("Redirecting to: {}", auth_url);

    leptos_axum::redirect(auth_url.as_str());
    Ok(())
}
