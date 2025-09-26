use super::*;

use basket_ordering::basket_state::client::refresh_basket_state_info_action;

use error_template::ErrorTemplate;

use leptos_meta::Title;
use stylers::style_sheet;

#[component]
pub fn OrdersPage() -> impl IntoView {
    use futures::{StreamExt, channel::mpsc};
    let (_tx, rx) = mpsc::channel(1);
    let refresh_orders = RwSignal::new(true);

    if cfg!(feature = "hydrate") {
        task::spawn_local(async move {
            match basket_ordering::basket::server_api::basket_state_notify(rx.into()).await {
                Ok(mut messages) => {
                    while let Some(_msg) = messages.next().await {
                        let old_val = refresh_orders.get_untracked();
                        refresh_orders.set(!old_val);
                    }
                }
                Err(e) => {
                    leptos::logging::error!("{e}");
                    let old_val = refresh_orders.get_untracked();
                    refresh_orders.set(!old_val);
                }
            }
        });
    }

    Effect::new(move || {
        refresh_basket_state_info_action().dispatch(());
    });

    let orders = Resource::new(move || refresh_orders.get(), |_| async move { basket_ordering::ordering::server_api::get_orders().await });

    let class_name = style_sheet!("./app/src/pages/orders/orders.css");

    let orders_view = move || {
        Suspend::new(async move {
            view! { class=class_name,
                {orders
                    .await
                    .map(|d| {
                        if d.is_empty() {
                            view! { class=class_name, <p>"You haven't yet placed any orders."</p> }
                                .into_any()
                        } else {
                            view! { class=class_name,
                                <ul class="orders-list">
                                    <li class="orders-header orders-item">
                                        <div>{"Number"}</div>
                                        <div>{"Date"}</div>
                                        <div class="total-header">{"Total"}</div>
                                        <div>{"Status"}</div>
                                    </li>
                                    {d
                                        .into_iter()
                                        .map(|item| {
                                            view! { class=class_name,
                                                <li class="orders-item">
                                                    <div class="order-number">{item.order_number}</div>
                                                    <div class="order-date">
                                                        {item.date.format("%d-%b-%y %H:%M:%S").to_string()}
                                                    </div>
                                                    <div class="order-total">
                                                        {format!("${:.2}", (item.total))}
                                                    </div>
                                                    <div class="order-status">
                                                        <span class=format!(
                                                            "status {class_name} {}",
                                                            item.status.clone().to_lowercase(),
                                                        )>{item.status.clone()}</span>
                                                    </div>
                                                </li>
                                            }
                                                .into_any()
                                        })
                                        .collect::<Vec<_>>()
                                        .into_any()}
                                </ul>
                            }
                                .into_any()
                        }
                    })
                    .into_any()}
            }
            .into_any()
        })
    };

    crate::app::page_header::set_title("Orders");

    view! { class=class_name,
        <Title text=format!("Orders | AdventureWorks") />
        <Suspense fallback=move || view! { <p>"Loading data..."</p> }>
            <ErrorBoundary fallback=|errors| view! { <ErrorTemplate errors /> }>
                <div class="orders">{orders_view}</div>
            </ErrorBoundary>
        </Suspense>
    }
}
