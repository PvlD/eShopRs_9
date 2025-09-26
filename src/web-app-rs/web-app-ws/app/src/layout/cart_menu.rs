use leptos::prelude::*;
use stylers::style_sheet;

use basket_ordering::basket_state::client::{BasketStateInfo, BasketStateInfoContext}; // Replace with actual types defined in basket_state

#[component]
pub fn CartMenu() -> impl IntoView {
    Effect::new(move || {
        Action::new(move |_: &()| {
            let context = expect_context::<BasketStateInfoContext>();
            let auth_context = expect_context::<auth::client::UserInfoCntxt>();
            basket_ordering::basket_state::client::refresh_basket_state_info(context, auth_context)
        })
        .dispatch(());
    });

    let basket_items = expect_context::<BasketStateInfoContext>().0;

    let basket_items_view = move || match basket_items.get() {
        BasketStateInfo { basket_items: Result::Ok(Some(items)) } if !items.is_empty() => {
            let items_count = items.iter().map(|item| item.quantity).sum::<i32>();
            Ok(view! { <span class="cart-badge">{items_count}</span> }
            .into_any())
        }

        BasketStateInfo { basket_items: Result::Err(e) } => Err(e),

        _ => Ok(().into_any()),
    };

    let class_name = style_sheet!("./app/src/layout/cart_menu.css");

    view! { class=class_name,
        <a aria-label="cart" href="cart">
            <img role="presentation" src="icons/cart.svg" />
            {basket_items_view}
        </a>
    }
}
