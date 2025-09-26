use crate::services::product_image_url_provider::ProductImageUrlContext;

use super::*;
use rust_decimal::prelude::*;

use basket_ordering::basket_state::{self, client::refresh_basket_state_info_action};
use leptos::ev::SubmitEvent;

use leptos_meta::Title;
use stylers::style_sheet;

use error_template::ErrorTemplate;

use std::ops::Mul;

#[component]
pub fn CartPage() -> impl IntoView {
    let basket_state_changed_signal = RwSignal::new(Ok(true) as Result<bool, crate::AppError>);

    let basket_items_res_signal = RwSignal::new(Ok(true) as Result<bool, crate::AppError>);
    let basket_items_res = Resource::new(
        move || basket_items_res_signal.get(),
        |s| async move {
            match s {
                Ok(_) => basket_state::server_api::get_basket_items().await,
                Err(e) => Err(e) as Result<Vec<basket_ordering::basket_state::types::BasketItem>, crate::AppError>,
            }
        },
    );

    Effect::new(move || match basket_state_changed_signal.get() {
        Ok(true) => {
            basket_items_res_signal.update(|s| match s {
                Ok(v) => {
                    *s = Ok(!*v);
                }
                Err(_) => {
                    *s = Ok(true);
                }
            });
        }
        Err(e) => basket_items_res_signal.set(Err(e)),
        _ => (),
    });

    let basket_state_set_set_quantity_action = Action::new(move |&(product_id, quantity): &(i32, i32)| async move { basket_state::server_api::set_quantity(product_id, quantity).await });

    Effect::new(move || match basket_state_set_set_quantity_action.value().get() {
        Some(Ok(true)) => {
            basket_state_changed_signal.set(Ok(true));
            refresh_basket_state_info_action().dispatch(());
        }
        Some(Err(e)) => basket_state_changed_signal.set(Err(e)),
        _ => (),
    });

    let class_name = style_sheet!("./app/src/pages/cart/cart.css");

    let cart_view = move || {
        Suspend::new(async move {
            basket_items_res.await.map(|basket_items| {
                let product_image_url_context = expect_context::<ProductImageUrlContext>();

                if basket_items.is_empty() {
                    view! { class=class_name, <p>Your shopping bag is empty. <a href="">Continue shopping.</a></p> }
                    .into_any()
                } else {
                    let total_quantity = basket_items.iter().map(|item| item.quantity).sum::<i32>();
                    let total_price = basket_items.iter().map(|item| item.unit_price * Decimal::from(item.quantity)).sum::<Decimal>();

                    view! { class=class_name,
                        <div class="cart-items">
                            <div class="cart-item-header">
                                <div class="catalog-item-info">Products</div>
                                <div class="catalog-item-quantity">Quantity</div>
                                <div class="catalog-item-total">Total</div>
                            </div>
                            {view! { class=class_name,
                                <For
                                    each=move || basket_items.clone()
                                    key=|item| item.product_id
                                    children=move |item| {
                                        let update_quantity_id = item.product_id;

                                        view! { class=class_name,
                                            <div class="cart-item">
                                                <div class="catalog-item-info">
                                                    <img
                                                        alt=item.product_name.clone()
                                                        src=product_image_url_context
                                                            .service
                                                            .get_product_image_url_by_id(item.product_id)
                                                    />
                                                    <div class="catalog-item-content">
                                                        <p class="name">{item.product_name.clone()}</p>
                                                        <p class="price">
                                                            {format!("${:.2}", item.unit_price.clone())}
                                                        </p>
                                                    </div>
                                                </div>
                                                <div class="catalog-item-quantity">
                                                    <form
                                                        method="post"
                                                        on:submit=move |ev: SubmitEvent| {
                                                            ev.prevent_default();
                                                            let target = wasm_bindgen::JsCast::unchecked_into::<
                                                                web_sys::HtmlFormElement,
                                                            >(ev.target().unwrap());
                                                            let form_data = web_sys::FormData::new_with_form(&target)
                                                                .unwrap();
                                                            let update_quantity_value: i32 = web_sys::js_sys::Number::from(
                                                                    form_data.get("UpdateQuantityValue"),
                                                                )
                                                                .value_of() as i32;
                                                            basket_state_set_set_quantity_action
                                                                .dispatch((update_quantity_id, update_quantity_value));
                                                        }
                                                    >

                                                        <input
                                                            aria-label="product quantity"
                                                            type="number"
                                                            name="UpdateQuantityValue"
                                                            value=item.quantity
                                                            min="0"
                                                        />
                                                        <button
                                                            type="submit"
                                                            class="button button-secondary"
                                                            name="UpdateQuantityId"
                                                            value=item.product_id
                                                        >
                                                            Update
                                                        </button>
                                                    </form>
                                                </div>
                                                <div class="catalog-item-total">
                                                    {format!(
                                                        "${:.2}",
                                                        item
                                                            .unit_price
                                                            .mul(rust_decimal::Decimal::from(item.quantity)),
                                                    )}
                                                </div>
                                            </div>
                                        }
                                    }
                                />
                            }
                                .into_any()}
                        </div>

                        <div class="cart-summary">
                            <div class="cart-summary-container">
                                <div class="cart-summary-header">
                                    <img role="presentation" src="icons/cart.svg" />
                                    Your shopping bag
                                    <span class="filter-badge">{total_quantity}</span>
                                </div>
                                <div class="cart-summary-total">
                                    <div>Total</div>
                                    <div>{format!("${:.2}", total_price)}</div>
                                </div>
                                <a href="checkout" class="button button-primary">
                                    Check out
                                </a>
                                <a href="" class="cart-summary-link">
                                    <img role="presentation" src="icons/arrow-left.svg" />
                                    <p>Continue shopping</p>
                                </a>
                            </div>
                        </div>
                    }
                    .into_any()
                }
            })
        })
    };

    view! { class=class_name,
        <Title text=format!("Shopping Bag | AdventureWorks") />
        <Transition fallback=move || view! { <p>"Loading data..."</p> }>
            <ErrorBoundary fallback=|errors| view! { <ErrorTemplate errors /> }>
                <div class="cart">{cart_view}</div>
            </ErrorBoundary>
        </Transition>
    }
}
