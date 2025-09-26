use std::sync::Arc;

use axum::http::Extensions;
// Import Router
// Import post function

// Import AxumBody
use leptos::prelude::ServerFnErrorErr;
// Import IntoResponse trait
use async_trait::async_trait;
use leptos_axum::extract;

use crate::service::{AuthService, AuthServiceContext, UserAddressInfo};
use crate::user::UserInfo;
use crate::users;
use anyhow::Result;

pub async fn token_from_auth_session() -> Result<String, crate::AppError> {
    let extensions: Extensions = extract().await?;
    let auth_session = extensions.get::<users::AuthSession>().ok_or(crate::AppError::Unauthorized)?;
    let t = auth_session.user.as_ref().ok_or(crate::AppError::Unauthorized)?.access_token.clone();
    Ok(t)
}

pub fn get_user_ref_from_extensions(extensions: &Extensions) -> Result<&users::User, crate::AppError> {
    let auth_session = if let Some(auth_session) = extensions.get::<users::AuthSession>() {
        auth_session
    } else {
        return Err(crate::AppError::Unauthorized);
    };
    if let Some(_user) = auth_session.user.as_ref() {
        return Ok(_user);
    } else {
        Err(crate::AppError::Unauthorized)
    }
}

pub async fn is_authenticated() -> Result<bool, ServerFnErrorErr> {
    let extensions: Extensions = extract().await?;
    let auth_session = if let Some(auth_session) = extensions.get::<users::AuthSession>() {
        auth_session
    } else {
        return Ok(false);
    };
    if let Some(_user) = auth_session.user.as_ref() {
        return Ok(true);
    } else {
        Ok(false)
    }
}

pub fn is_authenticated_from_extensions(extensions: &Extensions) -> Result<bool, ServerFnErrorErr> {
    let auth_session = if let Some(auth_session) = extensions.get::<users::AuthSession>() {
        auth_session
    } else {
        return Ok(false);
    };
    if let Some(_user) = auth_session.user.as_ref() {
        return Ok(true);
    } else {
        Ok(false)
    }
}

struct AuthServiceApi {}

impl AuthServiceApi {
    pub fn new() -> Self {
        AuthServiceApi {}
    }
}

#[async_trait]
impl AuthService for AuthServiceApi {
    async fn is_logged_in(&self) -> Result<Option<UserInfo>, crate::AppError> {
        let extensions: Extensions = extract().await?;
        let auth_session = if let Some(auth_session) = extensions.get::<users::AuthSession>() {
            auth_session
        } else {
            return Ok(None);
        };
        if let Some(_user) = auth_session.user.as_ref() {
            return Ok(Some(UserInfo {
                name: _user.username.clone(),
                email: _user.email.clone(),
            }));
        } else {
            return Ok(None);
        };
    }

    async fn get_user_address_info(&self) -> Result<UserAddressInfo, crate::AppError> {
        let extensions: Extensions = extract().await?;
        let auth_session = if let Some(auth_session) = extensions.get::<users::AuthSession>() {
            auth_session
        } else {
            return Ok(UserAddressInfo {
                street: None,
                city: None,
                state: None,
                zip: None,
                country: None,
            });
        };

        //log!("auth_session: {:#?}", auth_session);
        if let Some(_user) = auth_session.user.as_ref() {
            return Ok(UserAddressInfo {
                street: _user.street.clone(),
                city: _user.city.clone(),
                state: _user.state.clone(),
                zip: _user.zip.clone(),
                country: _user.country.clone(),
            });
        } else {
            return Ok(UserAddressInfo {
                street: None,
                city: None,
                state: None,
                zip: None,
                country: None,
            });
        };
    }
    async fn get_buyer_id(&self) -> Result<String, crate::AppError> {
        let extensions: Extensions = extract().await?;
        let user = crate::server::get_user_ref_from_extensions(&extensions)?;

        return Ok(user.sub.clone());
    }
    async fn get_user_name(&self) -> Result<String, crate::AppError> {
        let extensions: Extensions = extract().await?;
        let user = crate::server::get_user_ref_from_extensions(&extensions)?;
        return Ok(user.username.clone());
    }
}

pub fn make_service() -> Result<AuthServiceContext> {
    Ok(AuthServiceContext { service: Arc::new(AuthServiceApi::new()) })
}
