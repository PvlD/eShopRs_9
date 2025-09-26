use crate::client::UserInfoCntxt;

use super::super::super::*;
use super::sfn::*;

#[component]
pub fn Logout() -> impl IntoView {
    let user_info_context = expect_context::<UserInfoCntxt>().0;

    let logout_res = Resource::new(move || (), |_| logout());

    view! {
        <Suspense fallback=move || view! { <p>"Logout User"</p> }>
            <ErrorBoundary fallback=|errors| view! { <ErrorTemplate errors /> }>
                <p>
                    {move || {
                        logout_res
                            .get()
                            .map(|x| {
                                leptos::logging::log!("logout_res.get().map: {:?}  ", x);
                                match x {
                                    Ok(()) => {
                                        user_info_context.set(None);
                                        Ok("".to_string())
                                    }
                                    Err(e) => {
                                        log!("Logout error {}", e);
                                        Err(e)
                                    }
                                }
                            })
                    }}
                </p>

            </ErrorBoundary>
        </Suspense>
    }
}
