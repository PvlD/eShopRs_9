use components::{Outlet, ParentRoute, Route};
//use leptos::*;
//use leptos::prelude::*;
use leptos::*;
use leptos::{logging::log, prelude::*};

use leptos_router::*;

use error_template::ErrorTemplate;

pub mod utl;

pub mod user;

pub mod client;

#[cfg(feature = "ssr")]
pub mod openid_client;

#[cfg(feature = "ssr")]
pub mod users;

#[cfg(feature = "ssr")]
mod authorised;
#[cfg(feature = "ssr")]
pub use authorised::RequireAuth;

#[cfg(feature = "ssr")]
pub mod server;

#[cfg(feature = "ssr")]
pub mod require_auth;

use user::UserInfo;

use serde::{Deserialize, Serialize};

pub mod loginout;

pub mod server_api;

pub mod service;

pub use app_err::AppError;

pub const NEXT_URL_KEY: &str = "auth.next-url";
pub const CSRF_STATE_KEY: &str = "oauth.csrf-state";
pub const NONCE_STATE_KEY: &str = "oauth.nonce";

#[cfg(feature = "ssr")]
fn internal_server_error() -> crate::AppError {
    crate::AppError::ServerFnError(ServerFnErrorErr::ServerError("internal server error".to_string()))
}

fn urlstring_from_str_or_root(s: &str) -> String {
    match utl::urlstring_from_str(s) {
        Ok(url) => url,
        Err(e) => {
            leptos::logging::error!("urlstring_from_str_or_root Err:  {:?} ", e);
            "/".to_string()
        }
    }
}

pub fn login_url(url: &location::Url) -> String {
    let fn_path_from_url = || {
        let mut path = url.path().to_string();
        if !url.search().is_empty() {
            path.push('?');
            path.push_str(url.search());
        }
        if !url.hash().is_empty() {
            path.push('#');
            path.push_str(url.hash());
        }
        path
    };

    let login_url = format!("/user/login?nexturl={}", fn_path_from_url());
    login_url
}

pub fn login_url_from_current_url() -> String {
    let url: ReadSignal<location::Url> = hooks::use_url();
    let url = url.get_untracked();
    login_url(&url)
}

#[component]
pub fn UserAuthRoutes() -> impl MatchNestedRoutes + Clone {
    // TODO should make this mode configurable via feature flag?

    view! {
        <ParentRoute
            path=path!("/user")
            ssr=SsrMode::Async
            view=move || {
                view! { <Outlet /> }
            }
        >

            <Route
                path=path!("/login")
                ssr=SsrMode::Async
                view=move || view! { <loginout::Login /> }
            />

            <Route
                path=StaticSegment("/logout")
                ssr=SsrMode::Async
                view=move || view! { <loginout::Logout /> }
            />

            <Route
                path=StaticSegment("/signin-oidc")
                // ssr=SsrMode::Async
                view=move || view! { <loginout::OidcCallback /> }
            />

        </ParentRoute>
    }
    .into_inner()
}
