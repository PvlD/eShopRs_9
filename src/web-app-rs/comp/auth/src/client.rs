use crate::UserInfo;
use leptos::prelude::*;

pub type UserSigData = Option<UserInfo>;

#[derive(Clone, Debug)]
pub struct UserInfoCntxt(pub RwSignal<UserSigData>);

impl UserInfoCntxt {
    pub fn is_logged_in(&self) -> bool {
        self.0.get_untracked().is_some()
    }
}

pub fn init() -> Effect<LocalStorage> {
    let sig_rw_user_info = RwSignal::new(None as UserSigData);
    provide_context(UserInfoCntxt(sig_rw_user_info));

    #[cfg(feature = "hydrate")]
    {
        let is_logged_in_action = Action::new(move |_: &()| async move {
            let is_loged_in = crate::server_api::is_logged_in().await;
            match is_loged_in {
                Ok(Some(user_info)) => {
                    sig_rw_user_info.set(Some(user_info));
                }
                Ok(None) => {
                    sig_rw_user_info.set(None);
                }
                Err(e) => {
                    leptos::logging::error!("is_logged_in_action: {:?}  ", e);
                    sig_rw_user_info.set(None);
                }
            }
        });

        is_logged_in_action.dispatch(());
    }

    Effect::watch(
        move || sig_rw_user_info.get(),
        move |value, prev_val, _| {
            leptos::logging::log!("lgoff_effect: {:?}  {:?}  ", value, prev_val);
            if value.is_none() {
                let navigate = leptos_router::hooks::use_navigate();
                navigate("/", leptos_router::NavigateOptions::default());
            }
        },
        false,
    )
}

pub fn is_logged_in() -> bool {
    let user_info_context = expect_context::<UserInfoCntxt>();
    user_info_context.is_logged_in()
}
