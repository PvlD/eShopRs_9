use super::super::super::*;
use super::sfn::*;

use hooks::use_query;

use leptos_router::params::Params;

use thiserror::Error;
#[derive(PartialEq, Debug, Clone, Error)]
enum Error {
    #[error(transparent)]
    ParamsError(#[from] params::ParamsError),
}

impl From<Error> for AppError {
    fn from(value: Error) -> Self {
        crate::AppError::Other(value.to_string())
    }
}

#[derive(Params, Debug, PartialEq, Clone)]
struct NextUrlParams {
    pub nexturl: Option<String>,
}

#[component]
pub fn Login() -> impl IntoView {
    let params = use_query::<NextUrlParams>();
    let sig_params = Signal::derive(move || {
        params.with(|params| {
            params
                .as_ref()
                .map(|params: &NextUrlParams| match params {
                    NextUrlParams { nexturl: Some(nexturl) } => Ok(nexturl.to_string()),
                    _ => Ok("/".to_string()),
                })
                .map_err(|e| Error::ParamsError(e.clone()))
                .flatten()
        })
    });

    let login = Resource::new(
        move || sig_params(),
        |params| async move {
            let nexturl = params?;
            login(nexturl).await
        },
    );

    let login_view = move || Suspend::new(async move { login.await.map(|_| Ok::<(), crate::AppError>(())) });

    view! {
        <Suspense fallback=move || view! { <p>"Login User"</p> }>
            <ErrorBoundary fallback=|errors| {
                view! { <ErrorTemplate errors /> }
            }>{login_view}</ErrorBoundary>
        </Suspense>
    }
}
