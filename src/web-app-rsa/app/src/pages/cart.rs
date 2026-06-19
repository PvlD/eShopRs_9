use crate::auth::{get_auth_session, login_url};
use crate::layout::{BasketState, HeaderContext};
use crate::services::basket_state::set_basket_quantity;
use crate::models::BasketItem;
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_meta::Title;
use leptos_router::hooks::use_location;

#[component]
pub fn CartPage() -> impl IntoView {
    if let Some(ctx) = use_context::<HeaderContext>() {
        ctx.title.set("Shopping bag".to_string());
        ctx.subtitle.set(String::new());
    }

    let basket_state = use_context::<BasketState>();
    let auth = Resource::new(|| (), |_| async move { get_auth_session().await.ok() });
    let location = use_location();

    Effect::new(move |_| {
        if let Some(auth_session) = auth.get() {
            let is_auth = auth_session
                .map(|s| s.is_authenticated)
                .unwrap_or(false);
            if !is_auth {
                let href = login_url(&location.pathname.get());
                let _ = window().location().set_href(&href);
            }
        }
    });

    let items = move || {
        basket_state.and_then(|bs| bs.items.get())
    };

    let total_qty = move || {
        items().map(|ref items| items.iter().map(|i| i.quantity).sum::<i32>())
    };

    let total_price = move || {
        items().map(|ref items| items.iter().map(|i| i.quantity as f64 * i.unit_price).sum::<f64>())
    };

    view! {
        <Title text="Shopping Bag | AdventureWorks"/>
        <div class="cart">
            {move || match items() {
                None => ().into_any(),
                Some(ref items) if items.is_empty() => view! {
                    <p>"Your shopping bag is empty. " <a href="/">"Continue shopping."</a></p>
                }.into_any(),
                Some(ref items) => {
                    let items = items.clone();
                    view! {
                        <div class="cart-items">
                            <div class="cart-item-header">
                                <div class="catalog-item-info">"Products"</div>
                                <div class="catalog-item-quantity">"Quantity"</div>
                                <div class="catalog-item-total">"Total"</div>
                            </div>
                            {items.iter().map(|item| {
                                view! {
                                    <CartItemRow item=item.clone() />
                                }
                            }).collect::<Vec<_>>()}
                        </div>
                        <div class="cart-summary">
                            <div class="cart-summary-container">
                                <div class="cart-summary-header">
                                    <img role="presentation" src="/icons/cart.svg" />
                                    "Your shopping bag"
                                    <span class="filter-badge">{move || total_qty().unwrap_or(0)}</span>
                                </div>
                                <div class="cart-summary-total">
                                    <div>"Total"</div>
                                    <div>{move || total_price().map(|p| format!("${:.2}", p)).unwrap_or_else(|| "$0.00".to_string())}</div>
                                </div>
                                <a href="checkout" class="button button-primary">"Check out"</a>
                                <a href="/" class="cart-summary-link">
                                    <img role="presentation" src="/icons/arrow-left.svg" />
                                    <p>"Continue shopping"</p>
                                </a>
                            </div>
                        </div>
                    }.into_any()
                }
            }}
        </div>
    }
}

#[component]
fn CartItemRow(item: BasketItem) -> impl IntoView {
    let basket_state = use_context::<BasketState>().expect("BasketState not provided");
    let qty = RwSignal::new(item.quantity);
    let updating = RwSignal::new(false);
    let product_id = item.product_id;
    let img_src = format!("/product-images/{}?api-version=2.0", product_id);

    let on_update = move |_| {
        updating.set(true);
        let bs = basket_state;
        spawn_local(async move {
            let _ = set_basket_quantity(product_id, qty.get_untracked()).await;
            bs.refresh().await;
            updating.set(false);
        });
    };

    view! {
        <div class="cart-item">
            <div class="catalog-item-info">
                <img alt=item.product_name.clone() src=img_src />
                <div class="catalog-item-content">
                    <p class="name">{item.product_name.clone()}</p>
                    <p class="price">{format!("${:.2}", item.unit_price)}</p>
                </div>
            </div>
            <div class="catalog-item-quantity">
                <input
                    type="number"
                    prop:value=move || qty.get()
                    on:input=move |ev| {
                        if let Ok(v) = event_target_value(&ev).parse::<i32>() {
                            qty.set(v);
                        }
                    }
                    min="0"
                    disabled=move || updating.get()
                    aria-label="product quantity"
                />
                <button
                    class="button button-secondary"
                    on:click=on_update
                    disabled=move || updating.get()
                >
                    "Update"
                </button>
            </div>
            <div class="catalog-item-total">
                {move || format!("${:.2}", qty.get() as f64 * item.unit_price)}
            </div>
        </div>
    }
}
