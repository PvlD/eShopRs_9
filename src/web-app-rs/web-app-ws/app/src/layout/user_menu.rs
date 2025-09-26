use stylers::style_sheet;

use auth::{client::UserInfoCntxt, login_url};

use super::*;
use leptos_router::hooks::use_url;
use leptos_router::location;

#[component]
pub fn UserMenu() -> impl IntoView {
    let user_info_context = expect_context::<UserInfoCntxt>().0;
    let log_in_url = RwSignal::new("".to_string());
    let url: ReadSignal<location::Url> = use_url();

    Effect::new(move || {
        let url = url.get();
        log_in_url.set(login_url(&url));
    });

    let class_name = style_sheet!("./app/src/layout/user_menu.css");

    {
        move || {
            if user_info_context.get().is_none() {
                view! { class=class_name,
                    <a aria-label="Sign in" href=log_in_url>
                        <img role="presentation" src="icons/user.svg" />
                    </a>
                }
                .into_any()
            } else {
                view! { class=class_name,
                    <h3>{user_info_context.get().as_ref().unwrap().name.clone()}</h3>
                    <div class="dropdown-menu">
                        <span class="dropdown-button">
                            <img role="presentation" src="icons/user.svg" />
                        </span>
                        <div class="dropdown-content">
                            <a class="dropdown-item" href="user/orders">
                                My orders
                            </a>
                            <a class="dropdown-item" href="user/logout">
                                Log out
                            </a>
                        </div>
                    </div>
                }
                .into_any()
            }
        }
    }
}

//
