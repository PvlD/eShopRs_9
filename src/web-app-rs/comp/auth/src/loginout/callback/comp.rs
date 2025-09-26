use super::super::super::*;
use super::sfn::*;

use hooks::{use_navigate, use_query};

use crate::AppError;
use crate::client::UserInfoCntxt;
use leptos_router::params::Params;
use thiserror::Error;

#[derive(PartialEq, Debug, Clone, Error)]
enum CallbackError {
    #[error("Missing code or state parameter in OAuth callback URL")]
    MissingCodeOrState,
    #[error(transparent)]
    ParamsError(#[from] params::ParamsError),
}

#[derive(Params, Debug, PartialEq, Clone)]
struct OAuthParams {
    pub code: Option<String>,
    pub state: Option<String>,
}

impl From<CallbackError> for AppError {
    fn from(value: CallbackError) -> Self {
        AppError::Other(value.to_string())
    }
}

#[component]
pub fn OidcCallback() -> impl IntoView {
    let user_info_context = expect_context::<UserInfoCntxt>().0;

    let next_signal = RwSignal::new(None::<String>);

    Effect::new(move |_| {
        next_signal.get().map(|next_url| {
            let navigate = use_navigate();
            let next_url = crate::urlstring_from_str_or_root(&next_url);
            navigate(&next_url, Default::default());
        });
    });

    let params = use_query::<OAuthParams>();
    let sig_params = Signal::derive(move || {
        params.with(|params| {
            params
                .as_ref()
                .map(|params: &OAuthParams| match params {
                    OAuthParams { code: Some(code), state: Some(state) } => Ok((state.to_string(), code.to_string())),
                    _ => Err(CallbackError::MissingCodeOrState),
                })
                .map_err(|e| CallbackError::ParamsError(e.clone()))
                .flatten()
        })
    });

    let callback = Resource::new(
        move || sig_params(),
        |params| async move {
            let (state, code) = params?;
            handle_auth_redirect(state, code).await
        },
    );

    let callback_view = move || {
        Suspend::new(async move {
            callback.await.map(|(user_info, next_url)| {
                leptos::logging::log!("OK: {:?} ", user_info);
                user_info_context.set(Some(user_info));
                next_signal.set(Some(next_url));
                Ok::<String, AppError>("Success".to_string())
            })
        })
    };

    view! {
        <Suspense fallback=move || view! { <p>"Processing "</p> }>
            <ErrorBoundary fallback=move |errors| {
                view! { <ErrorTemplate errors /> }
            }>

                {callback_view}

            </ErrorBoundary>
        </Suspense>
    }
}
