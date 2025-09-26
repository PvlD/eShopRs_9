use super::super::super::*;

#[server(prefix = "/user")]
pub async fn handle_auth_redirect(provided_csrf: String, code: String) -> Result<(UserInfo, String), crate::AppError> {
    //return Err(internal_server_error());
    //log!("start handle_auth_redirect sfn code {:?} ", code);

    use axum_login::tower_sessions::Session;
    use leptos_axum::extract;
    use users::AuthSession;

    use users::Credentials;

    let mut auth_session: AuthSession = extract().await?;
    let session: Session = extract().await?;

    //    println!("Session {:?} ", session);

    let fn_err_log_session_key = move |key: &str| {
        leptos::logging::error!("session key {} missing ", key);
    };

    let Ok(Some(old_state)) = session.get(CSRF_STATE_KEY).await else {
        fn_err_log_session_key(CSRF_STATE_KEY);
        return Err(internal_server_error());
    };

    let Ok(Some(nonce)) = session.get(NONCE_STATE_KEY).await else {
        fn_err_log_session_key(NONCE_STATE_KEY);
        return Err(internal_server_error());
    };

    let creds = Credentials {
        code,
        old_state,
        new_state: openidconnect::CsrfToken::new(provided_csrf),
        nonce,
    };

    //println!("sfn old_state {:?} ", creds.old_state.secret());

    let user = match auth_session.authenticate(creds).await {
        Ok(Some(user)) => user,
        Ok(None) => {
            leptos::logging::error!("authenticate failed  ");
            return Err(internal_server_error());
        }
        Err(e) => {
            leptos::logging::error!("authenticate failed {:?} ", e);
            return Err(internal_server_error());
        }
    };

    auth_session.login(&user).await.map_err(|err| {
        leptos::logging::error!("{:?} ", err);
        internal_server_error()
    })?;

    let next = if let Ok(Some(next)) = session.remove::<String>(NEXT_URL_KEY).await {
        match next.as_str() {
            "" => "/".to_string(),
            _ => next,
        }
    } else {
        "/".to_string()
    };

    //println!("sfn next {:?} ", next);
    // log!("end   handle_auth_redirect sfn next {:?} ", next);

    Ok((UserInfo { name: user.username, email: user.email }, next))
}
