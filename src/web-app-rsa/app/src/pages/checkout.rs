use crate::auth::{get_auth_session, login_url};
use crate::layout::{BasketState, HeaderContext};
use crate::models::BasketCheckoutInfo;
use crate::services::basket_state::{checkout, get_checkout_info};
use crate::validation::{ValidationErrors, ValidationMessage, ValidationSummary};
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_meta::Title;
use leptos_router::hooks::{use_location, use_navigate};

#[component]
pub fn CheckoutPage() -> impl IntoView {
    if let Some(ctx) = use_context::<HeaderContext>() {
        ctx.title.set("Checkout".to_string());
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

    let street = RwSignal::new(String::new());
    let city = RwSignal::new(String::new());
    let state = RwSignal::new(String::new());
    let zip_code = RwSignal::new(String::new());
    let country = RwSignal::new(String::new());
    let submitting = RwSignal::new(false);
    let errors = ValidationErrors::new();
    let errors_submit = errors.clone();

    let checkout_info = Resource::new(|| (), |_| async move { get_checkout_info().await.ok() });

    Effect::new(move |_| {
        if let Some(Some(info)) = checkout_info.get() {
            street.set(info.street);
            city.set(info.city);
            state.set(info.state);
            zip_code.set(info.zip_code);
            country.set(info.country);
        }
    });

    let e = errors.clone();
    let navigate = use_navigate();

    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        if submitting.get_untracked() {
            return;
        }
        let s = street.get_untracked();
        let c = city.get_untracked();
        let st = state.get_untracked();
        let z = zip_code.get_untracked();
        let co = country.get_untracked();
        submitting.set(true);

        let info = BasketCheckoutInfo {
            street: s,
            city: c,
            state: st,
            zip_code: z,
            country: co,
            card_number: None,
            card_holder_name: None,
            card_security_number: None,
            card_expiration: None,
            card_type_id: 1,
            buyer: None,
            request_id: String::new(),
        };

        if !errors_submit.validate(&info, BasketCheckoutInfo::field_label) {
            submitting.set(false);
            return;
        }

        let nav = navigate.clone();
        let basket_state = use_context::<BasketState>();
        let errors_for_spawn = errors_submit.clone();
        spawn_local(async move {
            let result = checkout(info).await;
            submitting.set(false);
            match result {
                Ok(()) => {
                    if let Some(bs) = basket_state {
                        bs.refresh().await;
                    }
                    nav("/user/orders", Default::default());
                }
                Err(e) => {
                    errors_for_spawn.add("", &e.0);
                }
            }
        });
    };

    view! {
        <Title text="Checkout | AdventureWorks"/>
        <div class="checkout">
            <form on:submit=on_submit>
                <div class="form">
                    <div class="form-section">
                        <h2>"Shipping address"</h2>
                        <label>
                            "Address"
                            <input
                                type="text"
                                prop:value=move || street.get()
                                on:input=move |ev| street.set(event_target_value(&ev))
                                prop:name=move || "Info.Street".to_string()
                                class:invalid=move || e.has_field_error("street")
                                aria-invalid=move || e.has_field_error("street").to_string()
                            />
                        <ValidationMessage field="street" errors=e.clone() />
                    </label>
                        <div class="form-group">
                            <div class="form-group-item">
                                <label>
                                    "City"
                                    <input
                                        type="text"
                                        prop:value=move || city.get()
                                        on:input=move |ev| city.set(event_target_value(&ev))
                                        prop:name=move || "Info.City".to_string()
                                        class:invalid=move || e.has_field_error("city")
                                        aria-invalid=move || e.has_field_error("city").to_string()
                                    />
                                <ValidationMessage field="city" errors=e.clone() />
                            </label>
                            </div>
                            <div class="form-group-item">
                                <label>
                                    "State"
                                    <input
                                        type="text"
                                        prop:value=move || state.get()
                                        on:input=move |ev| state.set(event_target_value(&ev))
                                        prop:name=move || "Info.State".to_string()
                                        class:invalid=move || e.has_field_error("state")
                                        aria-invalid=move || e.has_field_error("state").to_string()
                                    />
                                <ValidationMessage field="state" errors=e.clone() />
                            </label>
                            </div>
                            <div class="form-group-item">
                                <label>
                                    "Zip code"
                                    <input
                                        type="text"
                                        prop:value=move || zip_code.get()
                                        on:input=move |ev| zip_code.set(event_target_value(&ev))
                                        prop:name=move || "Info.ZipCode".to_string()
                                        class:invalid=move || e.has_field_error("zip_code")
                                        aria-invalid=move || e.has_field_error("zip_code").to_string()
                                    />
                                <ValidationMessage field="zip_code" errors=e.clone() />
                            </label>
                            </div>
                        </div>
                        <label>
                            "Country"
                            <input
                                type="text"
                                prop:value=move || country.get()
                                on:input=move |ev| country.set(event_target_value(&ev))
                                prop:name=move || "Info.Country".to_string()
                                class:invalid=move || e.has_field_error("country")
                                aria-invalid=move || e.has_field_error("country").to_string()
                            />
                        <ValidationMessage field="country" errors=e.clone() />
                    </label>
                    </div>
                    <div class="form-section">
                        <div class="form-buttons">
                            <a href="/cart" class="button button-secondary">
                                <img role="presentation" src="/icons/arrow-left.svg" />
                                "Back to the shopping bag"
                            </a>
                            <button class="button button-primary" type="submit" disabled=move || submitting.get()>
                                {move || if submitting.get() { "Placing order..." } else { "Place order" }}
                            </button>
                        </div>
                    </div>
                    <ValidationSummary errors=e.clone() />
                </div>
            </form>
        </div>
    }
}
