use crate::components::chatbot::Chatbot;
use crate::models::BasketItem;
use crate::services::basket_state::{add_basket_item, get_basket_items, set_basket_quantity};
use leptos::prelude::*;

use leptos_router::{components::Outlet, hooks::use_location};
use crate::auth::{get_auth_session, login_url};

#[derive(Clone, Copy)]
pub struct HeaderContext {
    pub title: RwSignal<String>,
    pub subtitle: RwSignal<String>,
}

#[derive(Clone, Copy)]
pub struct BasketState {
    pub items: RwSignal<Option<Vec<BasketItem>>>,
}

impl BasketState {
    pub async fn refresh(self) {
        match get_basket_items().await {
            Ok(items) => self.items.set(Some(items)),
            Err(_) => self.items.set(None),
        }
    }

    pub async fn add_item(self, product_id: i32) {
        let _ = add_basket_item(product_id).await;
        self.refresh().await;
    }

    pub async fn set_quantity(self, product_id: i32, quantity: i32) {
        let _ = set_basket_quantity(product_id, quantity).await;
        self.refresh().await;
    }
}

#[component]
pub fn MainLayout() -> impl IntoView {
    let header_ctx = HeaderContext {
        title: RwSignal::new("eShop".to_string()),
        subtitle: RwSignal::new("".to_string()),
    };
    provide_context(header_ctx);

    let basket_state = BasketState {
        items: RwSignal::new(None),
    };
    provide_context(basket_state);
    let _ = LocalResource::new(move || {
        basket_state.refresh()
    });

    view! {
        <HeaderBar />
        <main>
            <Outlet />
        </main>
        <ShowChatbotButton />
        <FooterBar />
        <div id="blazor-error-ui">
            "An unhandled error has occurred."
            <a href="" class="reload">"Reload"</a>
            <a class="dismiss">"🗙"</a>
        </div>
    }
}

#[component]
fn HeaderBar() -> impl IntoView {
    let location = use_location();
    let is_home = move || location.pathname.get() == "/";
    let header_class = move || {
        if is_home() {
            "eshop-header home"
        } else {
            "eshop-header"
        }
    };
    let header_img = move || {
        if is_home() {
            "/images/header-home.webp"
        } else {
            "/images/header.webp"
        }
    };

    let ctx = use_context::<HeaderContext>();
    let title = move || ctx.map(|c| c.title.get()).unwrap_or_default();
    let subtitle = move || ctx.map(|c| c.subtitle.get()).unwrap_or_default();

    view! {
        <div class=header_class>
            <div class="eshop-header-hero">
                <img role="presentation" src=header_img />
            </div>
            <div class="eshop-header-container">
                <nav class="eshop-header-navbar">
                    <a class="logo logo-header" href="/">
                        <img
                            alt="AdventureWorks"
                            src="/images/logo-header.svg"
                            class="logo logo-header"
                        />
                    </a>
                    <UserMenu />
                    <CartMenu />
                </nav>
                <div class="eshop-header-intro">
                    <h1>{title}</h1>
                    <p>{subtitle}</p>
                </div>
            </div>
        </div>
    }
}

#[component]
fn FooterBar() -> impl IntoView {
    view! {
        <footer class="eshop-footer">
            <div class="eshop-footer-content">
                <div class="eshop-footer-row">
                    <img role="presentation" src="/images/logo-footer.svg" class="logo logo-footer" />
                    <p>"© AdventureWorks"</p>
                </div>
            </div>
        </footer>
    }
}

#[component]
fn CartMenu() -> impl IntoView {
    let basket_state = use_context::<BasketState>();
    let total_qty = move || {
        basket_state
            .and_then(|bs| bs.items.get())
            .map(|items| items.iter().map(|i| i.quantity).sum::<i32>())
            .unwrap_or(0)
    };

    view! {
        <a aria-label="cart" href="/cart" style="position:relative;">
            <img role="presentation" src="/icons/cart.svg" />
            {move || (total_qty() > 0).then(|| view! { <span class="cart-badge">{total_qty()}</span> })}
        </a>
    }
}

#[component]
fn UserMenu() -> impl IntoView {
    let auth = Resource::new(|| (), |_| async move { get_auth_session().await.ok() });
    let location = use_location();

    view! {
        <Transition fallback=|| view! {}>
            {move || auth.get().map(|session| match session {
                Some(s) if s.is_authenticated => {
                    let user_name = s.user_name.clone().unwrap_or_default();
                    view! {
                        <h3>{user_name}</h3>
                        <div class="dropdown-menu">
                            <span class="dropdown-button">
                                <img role="presentation" src="/icons/user.svg" />
                            </span>
                            <div class="dropdown-content">
                                <a class="dropdown-item" href="/user/orders" rel="external">"My orders"</a>
                                <form class="dropdown-item" action="/user/logout" method="post">
                                    <button type="submit">"Log out"</button>
                                </form>
                            </div>
                        </div>
                    }.into_any()
                }
                _ => {
                    let loc = location.clone();
                    view! {
                        <a aria-label="Sign in" href=move || login_url(&loc.pathname.get()) rel="external">
                            <img role="presentation" src="/icons/user.svg" />
                        </a>
                    }.into_any()
                }
            })}
        </Transition>
    }
}

#[component]
fn ShowChatbotButton() -> impl IntoView {
    use leptos_router::hooks::use_query_map;
    let query_map = use_query_map();
    let show_chat = move || query_map.with(|p| p.get("chat").unwrap_or_default() == "true");

    view! {
        <a class="show-chatbot" href="?chat=true" title="Show chatbot"></a>
        {move || show_chat().then(|| view! { <Chatbot /> })}
    }
}
