use basket_ordering::basket_state::client::refresh_basket_state_info_action;
use leptos::ev::SubmitEvent;
use leptos_router::{NavigateOptions, hooks::use_navigate, location};
use stylers::style_sheet;

use crate::services::product_image_url_provider::ProductImageUrlContext;

use auth::client::UserInfoCntxt;
use catalog::{server_api::get_catalog_item, types::CatalogItem};
use error_template::ErrorTemplate;

use super::*;

use rust_decimal::prelude::ToPrimitive;

#[derive(Params, PartialEq, Clone, Debug)]
pub struct ItemIdParams {
    item_id: Option<usize>,
}

#[component]
pub fn ItemPage() -> impl IntoView {
    let params = use_params::<ItemIdParams>();

    let sig_item_id = Signal::derive(move || params.with(|item_id| item_id.as_ref().map(|d| d.item_id.ok_or(ItemPageError::InvalidItemIndex)).map_err(|_| ItemPageError::InvalidItemIndex).flatten()));

    let sig_num_in_cart = RwSignal::new(Ok(0) as Result<i32, crate::AppError>);

    let sig_basket_state_info = basket_ordering::basket_state::client::signal_from_context();

    let item = Resource::new(move || sig_item_id(), |item_id| async move { get_catalog_item(item_id?).await });

    let add_to_basket_action = Action::new(move |item: &CatalogItem| {
        let item = item.clone();
        async move { basket_ordering::basket_state::server_api::add_basket_item(item).await }
    });

    Effect::new(move || match add_to_basket_action.value().get() {
        Some(Ok(_)) => {
            refresh_basket_state_info_action().dispatch(());
        }
        Some(Err(e)) => sig_num_in_cart.set(Err(e)),
        None => (),
    });

    let class_name = style_sheet!("./app/src/pages/item/item.css");

    let num_in_cart_view = move || match sig_num_in_cart.get() {
        Ok(num) if num > 0 => Ok(view! { class=class_name,
            <p>
                <strong>{num}</strong>
                " in "
                <a href="cart">"shopping bag"</a>
            </p>
        }.into_any()),
        Err(e) => Err(e),
        _ => Ok({
            let _: () = view! {};
            ().into_any()
        }),
    };

    let item_view = move || {
        Suspend::new(async move {
            let product_image_url_context = expect_context::<ProductImageUrlContext>();

            let mut price_fmt = numfmt::Formatter::new() // start with blank representation
                .precision(numfmt::Precision::Decimals(2));

            let user_info_context = expect_context::<UserInfoCntxt>().0;
            let is_logged_in = user_info_context.get().is_some();

            item.await.map(|item: Option<catalog::types::CatalogItem>| {
                item.map_or_else(
                    || {
                        crate::app::page_header::set_title("Not found");
                        view! {
                            <div class="item-details">
                                <p>"Sorry, we couldn't find any such product."</p>
                            </div>
                        }
                        .into_any()
                    },
                    |item| {
                        Effect::new(move || match sig_basket_state_info.get().basket_item_by_id(item.id) {
                            Ok(Some(basket_items)) => {
                                sig_num_in_cart.set(Ok(basket_items.quantity));
                            }
                            Ok(None) => sig_num_in_cart.set(Ok(0)),
                            Err(e) => sig_num_in_cart.set(Err(e)),
                        });

                        let barand_name = item.catalog_brand.clone().map(|x| x.brand).unwrap_or_default();

                        let price = price_fmt.fmt2(item.price.to_f64().unwrap_or_default());

                        crate::app::page_header::set_title(item.name.to_string().as_str());
                        crate::app::page_header::set_subtitle(barand_name.as_str());

                        let product_image_url = product_image_url_context.service.get_product_image_url(&item);
                        let item_ = item.clone();
                        let on_submit = move |e: SubmitEvent| {
                            e.prevent_default();
                            match is_logged_in {
                                true => {
                                    add_to_basket_action.dispatch(item_.clone());
                                }
                                false => {
                                    let url: ReadSignal<location::Url> = hooks::use_url();
                                    let url = url.get_untracked();
                                    let navigate = use_navigate();
                                    let login_url = auth::login_url(&url);
                                    navigate(&login_url, NavigateOptions::default());
                                }
                            }
                        };

                        view! { class=class_name,
                            <Title text=format!("{} | AdventureWorks", item.name) />

                            <div class="item-details">
                                <img alt=item.name src=product_image_url />
                                <div class="description">
                                    <p>{item.description}</p>
                                    <p>Brand: <strong>{barand_name}</strong></p>
                                    <form
                                        class="add-to-cart"
                                        // method="post"
                                        on:submit=on_submit
                                    >

                                        <span class="price">{price.to_string()}</span>

                                        {move || {
                                            if is_logged_in {
                                                view! { class=class_name,
                                                    <button type="submit" title="Add to basket">
                                                        <svg
                                                            width="24"
                                                            height="24"
                                                            viewBox="0 0 24 24"
                                                            fill="none"
                                                            stroke="currentColor"
                                                            xmlns="http://www.w3.org/2000/svg"
                                                        >
                                                            <path
                                                                id="Vector"
                                                                d="M6 2L3 6V20C3 20.5304 3.21071 21.0391 3.58579 21.4142C3.96086 21.7893 4.46957 22 5 22H19C19.5304 22 20.0391 21.7893 20.4142 21.4142C20.7893 21.0391 21 20.5304 21 20V6L18 2H6Z"
                                                                stroke-width="1.5"
                                                                stroke-linecap="round"
                                                                stroke-linejoin="round"
                                                            />
                                                            <path
                                                                id="Vector_2"
                                                                d="M3 6H21"
                                                                stroke-width="1.5"
                                                                stroke-linecap="round"
                                                                stroke-linejoin="round"
                                                            />
                                                            <path
                                                                id="Vector_3"
                                                                d="M16 10C16 11.0609 15.5786 12.0783 14.8284 12.8284C14.0783 13.5786 13.0609 14 12 14C10.9391 14 9.92172 13.5786 9.17157 12.8284C8.42143 12.0783 8 11.0609 8 10"
                                                                stroke-width="1.5"
                                                                stroke-linecap="round"
                                                                stroke-linejoin="round"
                                                            />
                                                        </svg>
                                                        Add to shopping bag
                                                    </button>
                                                }
                                                    .into_any()
                                            } else {

                                                view! { class=class_name,
                                                    <button type="submit" title="Log in to purchase">
                                                        <svg
                                                            width="24"
                                                            height="24"
                                                            viewBox="0 0 24 24"
                                                            fill="none"
                                                            stroke="currentColor"
                                                            xmlns="http://www.w3.org/2000/svg"
                                                        >
                                                            <path
                                                                d="M20 21V19C20 17.9391 19.5786 16.9217 18.8284 16.1716C18.0783 15.4214 17.0609 15 16 15H8C6.93913 15 5.92172 15.4214 5.17157 16.1716C4.42143 16.9217 4 17.9391 4 19V21"
                                                                stroke-width="1.5"
                                                                stroke-linecap="round"
                                                                stroke-linejoin="round"
                                                            />
                                                            <path
                                                                d="M12 11C14.2091 11 16 9.20914 16 7C16 4.79086 14.2091 3 12 3C9.79086 3 8 4.79086 8 7C8 9.20914 9.79086 11 12 11Z"
                                                                stroke-width="1.5"
                                                                stroke-linecap="round"
                                                                stroke-linejoin="round"
                                                            />
                                                        </svg>
                                                        Log in to purchase
                                                    </button>
                                                }
                                                    .into_any()
                                            }
                                        }}

                                    </form>

                                    {num_in_cart_view}

                                </div>
                            </div>
                        }
                        .into_any()
                    },
                )
            })
        })
    };

    view! { class=class_name,
        <Title text="Northern Mountains" />
        <Suspense fallback=move || view! { <p>"Loading data..."</p> }>
            <ErrorBoundary fallback=|errors| view! { <ErrorTemplate errors /> }>

                <div>{item_view}</div>
            </ErrorBoundary>
        </Suspense>
    }
}
