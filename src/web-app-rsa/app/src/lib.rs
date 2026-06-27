use leptos::prelude::*;
use leptos_meta::{provide_meta_context, MetaTags, Stylesheet, Title};
use leptos_router::{
    components::{ParentRoute, Route, Router, Routes},
    ParamSegment, StaticSegment,
};

#[cfg(feature = "ssr")]
pub use settings;
pub mod auth;
pub mod components;
pub mod error;
pub mod layout;
pub mod models;
pub mod pages;
pub mod services;
pub mod validation;
pub use services::basket;
pub use services::basket_state;
pub use services::ordering;
use layout::MainLayout;
use pages::cart::CartPage;
use pages::catalog::CatalogPage;
use pages::checkout::CheckoutPage;
use pages::item::ItemPage;
use pages::orders::OrdersPage;
pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8"/>
                <meta name="viewport" content="width=device-width, initial-scale=1"/>
                <script>
                    {r#"
(function() {
    if (window.__opencode_ws_patched) return;
    window.__opencode_ws_patched = true;

    var OrigWS = WebSocket;
    WebSocket = function(url, protocols) {
        if (typeof url === 'string' && url.startsWith('wss://127.0.0.1:3031/')) {
            console.warn("[opencode] downgraded WSS to WS for 3031");
            url = url.replace("wss://", "ws://");
        }
        return new OrigWS(url, protocols);
    };
    WebSocket.prototype = OrigWS.prototype;
    WebSocket.CONNECTING = 0;
    WebSocket.OPEN = 1;
    WebSocket.CLOSING = 2;
    WebSocket.CLOSED = 3;
})();
"#}  
                </script>
                <AutoReload options=options.clone()/>
                <HydrationScripts options/>
                <MetaTags/>
            </head>
            <body>
                <App/>
            </body>
        </html>
    }
}

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    view! {
        <Stylesheet id="leptos" href="/pkg/webapprsa3.css"/>
        <Title text="eShop"/>
        <Router>
            <Routes fallback=|| "Page not found.".into_view() transition=true>
                <ParentRoute path=StaticSegment("") view=MainLayout>
                    <Route path=StaticSegment("") view=CatalogPage/>
                    <Route path=(StaticSegment("item"), ParamSegment("id")) view=ItemPage />
                    <Route path=StaticSegment("cart") view=CartPage/>
                    <Route path=StaticSegment("checkout") view=CheckoutPage  />
                    <Route path=(StaticSegment("user"), StaticSegment("orders")) view=OrdersPage/>
                </ParentRoute>
            </Routes>
        </Router>
    }
}
