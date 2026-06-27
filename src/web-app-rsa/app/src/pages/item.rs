use crate::auth::{get_auth_session, login_url};
use crate::layout::{BasketState, HeaderContext};
#[cfg(feature = "ssr")]
use http::StatusCode;
use crate::services::get_catalog_item;
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_meta::Title;
use leptos_router::hooks::{use_location, use_params_map};

#[component]
pub fn ItemPage() -> impl IntoView {
    let ctx = use_context::<HeaderContext>();
    let params = use_params_map();
    let location = use_location();
    let id = move || {
        params
            .get()
            .get("id")
            .and_then(|v| v.parse::<i32>().ok())
            .unwrap_or(0)
    };

    let auth = Resource::new(|| (), |_| async move { get_auth_session().await.ok() });

    let name = RwSignal::new(String::new());
    let brand = RwSignal::new(String::new());
    let description = RwSignal::new(String::new());
    let price = RwSignal::new(String::new());
    let product_id = RwSignal::new(0i32);
    let img_src = RwSignal::new(String::new());
    let show_not_found = RwSignal::new(false);
    let show_error = RwSignal::new(None::<String>);
    let logged_in = RwSignal::new(false);

    let item = Resource::new(id, |id| async move { get_catalog_item(id).await });

    Effect::new(move |_| {
        if let Some(result) = item.get() {
            match result {
                Ok(Some(catalog_item)) => {
                    if let Some(ctx) = ctx {
                        ctx.title.set(catalog_item.name.clone());
                        ctx.subtitle.set(
                            catalog_item
                                .catalog_brand
                                .as_ref()
                                .map(|b| b.brand.clone())
                                .unwrap_or_default(),
                        );
                    }
                    name.set(catalog_item.name.clone());
                    brand.set(catalog_item
                        .catalog_brand
                        .as_ref()
                        .map(|b| b.brand.clone())
                        .unwrap_or_default());
                    description.set(catalog_item.description.clone());
                    price.set(format!("${:.2}", catalog_item.price));
                    product_id.set(catalog_item.id);
                    img_src.set(format!("/product-images/{}?api-version=2.0", catalog_item.id));
                    show_not_found.set(false);
                    show_error.set(None);
                    let is_auth = auth.get().flatten().map(|s| s.is_authenticated).unwrap_or(false);
                    logged_in.set(is_auth);
                }
                Ok(None) => {
                    #[cfg(feature = "ssr")]
                    if let Some(resp) = use_context::<leptos_axum::ResponseOptions>() {
                        resp.set_status(StatusCode::NOT_FOUND);
                    }
                    if let Some(ctx) = ctx {
                        ctx.title.set("Not found".to_string());
                        ctx.subtitle.set(String::new());
                    }
                    show_not_found.set(true);
                }
                Err(e) => {
                    if let Some(ctx) = ctx {
                        ctx.title.set("Error".to_string());
                        ctx.subtitle.set(String::new());
                    }
                    show_error.set(Some(e.to_string()));
                }
            }
        }
    });

    let basket_state = use_context::<BasketState>();
    let href = move || login_url(&location.pathname.get());

    view! {
        <Title text={move || {
            if show_not_found.get() {
                "Not found | AdventureWorks".to_string()
            } else if show_error.get().is_some() {
                "Error | AdventureWorks".to_string()
            } else {
                format!("{} | AdventureWorks", name.get())
            }
        }}/>
        {move || {
            if show_not_found.get() {
                view! {
                    <div class="item-details">
                        <p>"Sorry, we couldn't find any such product."</p>
                    </div>
                }.into_any()
            } else if let Some(e) = show_error.get() {
                view! {
                    <p>"Error: " {e}</p>
                }.into_any()
            } else {
                let logged_in = logged_in.get();
                let product_id = product_id.get();
                let basket_state_add = basket_state;
                let basket_state_display = basket_state;
                let href = href.clone();
                view! {
                    <div class="item-details">
                        <img alt={move || name.get()} src={move || img_src.get()} />
                        <div class="description">
                            <p>{move || description.get()}</p>
                            <p>"Brand: " <strong>{move || brand.get()}</strong></p>
                            <div class="add-to-cart">
                                <span class="price">{move || price.get()}</span>
                                {if logged_in {
                                    view! {
                                        <button on:click=move |_| {
                                            let bs = basket_state_add;
                                            spawn_local(async move {
                                                if let Some(bs) = bs {
                                                    bs.add_item(product_id).await;
                                                }
                                            });
                                        } title="Add to basket">
                                            <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" xmlns="http://www.w3.org/2000/svg">
                                                <path d="M6 2L3 6V20C3 20.5304 3.21071 21.0391 3.58579 21.4142C3.96086 21.7893 4.46957 22 5 22H19C19.5304 22 20.0391 21.7893 20.4142 21.4142C20.7893 21.0391 21 20.5304 21 20V6L18 2H6Z" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" />
                                                <path d="M3 6H21" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" />
                                                <path d="M16 10C16 11.0609 15.5786 12.0783 14.8284 12.8284C14.0783 13.5786 13.0609 14 12 14C10.9391 14 9.92172 13.5786 9.17157 12.8284C8.42143 12.0783 8 11.0609 8 10" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" />
                                            </svg>
                                            "Add to shopping bag"
                                        </button>
                                    }.into_any()
                                } else {
                                    view! {
                                        <a href=href rel="external" title="Log in to purchase">
                                            <button type="button">
                                                <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" xmlns="http://www.w3.org/2000/svg">
                                                    <path d="M20 21V19C20 17.9391 19.5786 16.9217 18.8284 16.1716C18.0783 15.4214 17.0609 15 16 15H8C6.93913 15 5.92172 15.4214 5.17157 16.1716C4.42143 16.9217 4 17.9391 4 19V21" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" />
                                                    <path d="M12 11C14.2091 11 16 9.20914 16 7C16 4.79086 14.2091 3 12 3C9.79086 3 8 4.79086 8 7C8 9.20914 9.79086 11 12 11Z" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" />
                                                </svg>
                                                "Log in to purchase"
                                            </button>
                                        </a>
                                    }.into_any()
                                }}
                            </div>
                            {move || {
                                basket_state_display
                                    .and_then(|bs| bs.items.get())
                                    .map(|items| {
                                        let pid = product_id;
                                        let count = items
                                            .iter()
                                            .find(|bi| bi.product_id == pid)
                                            .map(|bi| bi.quantity)
                                            .unwrap_or(0);
                                        if count > 0 {
                                            view! {
                                                <p>
                                                    <strong>{count}</strong>
                                                    " in "
                                                    <a href="/cart">"shopping bag"</a>
                                                </p>
                                            }.into_any()
                                        } else {
                                            ().into_any()
                                        }
                                    })
                                    .unwrap_or_else(|| ().into_any())
                            }}
                        </div>
                    </div>
                }.into_any()
            }
        }}
    }
}
