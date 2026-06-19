use crate::auth::{get_auth_session, login_url};
use crate::layout::HeaderContext;
use crate::services::ordering::get_orders;
use leptos::prelude::*;
use leptos_meta::Title;
use leptos_router::hooks::use_location;

fn format_order_date(date_str: &str) -> String {
    let without_t = date_str.replace('T', " ");
    match without_t.find('.') {
        Some(pos) => without_t[..pos].to_string(),
        None => without_t.trim_end_matches('Z').to_string(),
    }
}

#[component]
pub fn OrdersPage() -> impl IntoView {
    if let Some(ctx) = use_context::<HeaderContext>() {
        ctx.title.set("Orders".to_string());
        ctx.subtitle.set(String::new());
    }

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

    let refresh = RwSignal::new(0u32);
    let orders = Resource::new(
        move || refresh.get(),
        |_| async move { get_orders().await },
    );

    #[cfg(feature = "hydrate")]
    {
        use wasm_bindgen::prelude::*;
        use web_sys::MessageEvent;

        let es = web_sys::EventSource::new("/api/orders/events")
            .expect("failed to create EventSource");
        let refresh_clone = refresh;
        let on_msg = Closure::<dyn FnMut(MessageEvent)>::new(move |_| {
            refresh_clone.update(|n| *n += 1);
        });
        es.set_onmessage(Some(on_msg.as_ref().unchecked_ref()));
        let closure = StoredValue::new_local(Some(on_msg));
        on_cleanup(move || {
            es.close();
            if let Some(mut g) = closure.try_write_value() {
                *g = None;
            }
        });
    }

    view! {
        <Title text="Orders | AdventureWorks"/>
        <div class="orders">
            <Transition fallback=|| ()>
                {move || orders.get().map(|result| match result {
                    Ok(list) if list.is_empty() => view! { <p>"You haven't yet placed any orders."</p> }.into_any(),
                    Ok(list) => {
                        view! {
                            <ul class="orders-list">
                                <li class="orders-item orders-header">
                                    <div>"Number"</div>
                                    <div>"Date"</div>
                                    <div class="total-header">"Total"</div>
                                    <div>"Status"</div>
                                </li>
                                {list.into_iter().map(|order| {
                                    let status_class = order.status.to_lowercase();
                                    view! {
                                        <li class="orders-item">
                                            <div class="order-number">{order.order_number}</div>
                                            <div class="order-date">{format_order_date(&order.date)}</div>
                                            <div class="order-total">{format!("${:.2}", order.total)}</div>
                                            <div class="order-status">
                                                <span class=format!("status {}", status_class)>
                                                    {order.status}
                                                </span>
                                            </div>
                                        </li>
                                    }
                                }).collect::<Vec<_>>()}
                            </ul>
                        }.into_any()
                    }
                    Err(_) => view! { <p>"Failed to load orders."</p> }.into_any(),
                })}
            </Transition>
        </div>
    }
}
