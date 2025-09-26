pub mod hlp;
pub mod layout;
pub mod pages;

pub(crate) mod edit_form;
pub mod services;

pub(crate) mod components;
//use components;

use layout::*;
use leptos::prelude::*;
use leptos_meta::{Stylesheet, Title, provide_meta_context};
use leptos_router::{
    StaticSegment,
    components::{Route, Router, Routes},
};
use pages::*;

#[cfg(feature = "ssr")]
use leptos_meta::MetaTags;

use hlp::section_outlet;

use auth::AppError;

pub use app::App;

#[cfg(feature = "ssr")]
pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8" />
                <meta name="viewport" content="width=device-width, initial-scale=1" />
                <base href="/" />
                <link rel="stylesheet" href="css/normalize.css" />
                <link rel="stylesheet" href="css/app.css" />

                <link rel="shortcut icon" href="images/favicon.png" />
                // required to load JS/WASM for client-side interactivity
                <HydrationScripts options=options.clone() />
                // optional: can be toggled to include/exclude cargo-leptos live-reloading code
                <AutoReload options=options.clone() />
                // required if using leptos_meta
                <MetaTags />
            </head>
            <body>
                <App />
            </body>
        </html>
    }
}

pub mod app {
    use leptos_router::{components::ProtectedRoute, path};

    use super::*;

    #[component]
    pub fn App() -> impl IntoView {
        // Provides context that manages stylesheets, titles, meta tags, etc.
        provide_meta_context();

        basket_ordering::basket_state::client::provide_basket_state_info_context();
        basket_ordering::basket::client::provide_basket_service_context();
        //basket_ordering::basket_state::client::provide_basket_state_service_context();

        let key_dispatcher = section_outlet::KeyDispatcher::new();
        key_dispatcher.provide_context();

        let (s_get_title, s_get_subtitle) = page_header::init();

        let product_image_url_context = services::product_image_url_provider::make_service();
        provide_context(product_image_url_context);

        let _logoff_effect = auth::client::init();

        view! {
            // injects a stylesheet into the document <head>
            // id=leptos means cargo-leptos will hot-reload this stylesheet
            <Stylesheet id="leptos" href="/pkg/web-app-ws.css" />

            // sets the document title
            <Title text="Welcome to Leptos" />

            // content for this welcome page
            <Router>
                <main>
                    <HeaderBar s_get_title=s_get_title s_get_subtitle=s_get_subtitle />
                    <Routes fallback=|| "Page not found.".into_view()>

                        <auth::UserAuthRoutes />

                        <Route path=StaticSegment("/") view=CatalogPage />
                        <Route path=path!("/item/:item_id") view=ItemPage />

                        <ProtectedRoute
                            path=path!("/cart")
                            view=CartPage
                            condition=move || Some(auth::client::is_logged_in())
                            redirect_path=|| auth::login_url_from_current_url()
                        />

                        <ProtectedRoute
                            path=path!("/checkout")
                            view=CheckoutPage
                            condition=move || Some(auth::client::is_logged_in())
                            redirect_path=|| auth::login_url_from_current_url()
                        />
                        <ProtectedRoute
                            path=path!("/user/orders")
                            view=OrdersPage
                            condition=move || Some(auth::client::is_logged_in())
                            redirect_path=|| auth::login_url_from_current_url()
                        />
                    </Routes>

                    <FooterBar />
                </main>
            </Router>
        }
    }

    /// Renders the home page of your application.
    #[component]
    fn HomePage() -> impl IntoView {
        // Creates a reactive value to update the button
        let count = RwSignal::new(0);
        let on_click = move |_| *count.write() += 1;

        view! {
            <h1>"Welcome to Leptos!"</h1>
            <button on:click=on_click>"Click Me: " {count}</button>
        }
    }

    pub(crate) mod page_header {
        use crate::hlp::section_outlet::KeyDispatcher;
        use leptos::prelude::{ReadSignal, expect_context};

        const PAGE_HEADER_TITLE: &str = "page-header-title";
        const PAGE_HEADER_SUBTITLE: &str = "page-header-subtitle";

        pub(super) fn init() -> (ReadSignal<String>, ReadSignal<String>) {
            let disp: KeyDispatcher = expect_context();
            let s_get_title = disp.register(PAGE_HEADER_TITLE);
            let s_get_subtitle = disp.register(PAGE_HEADER_SUBTITLE);
            (s_get_title, s_get_subtitle)
        }

        pub(crate) fn set_title(val: &str) {
            KeyDispatcher::set_value(PAGE_HEADER_TITLE, val);
        }

        pub(crate) fn set_subtitle(val: &str) {
            KeyDispatcher::set_value(PAGE_HEADER_SUBTITLE, val);
        }
    }
}
