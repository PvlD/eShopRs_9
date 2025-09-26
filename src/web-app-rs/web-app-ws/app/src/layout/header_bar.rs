use leptos::prelude::*;
use leptos_router::hooks::use_url;

use stylers::style_sheet;

use crate::layout::CartMenu;
use crate::layout::UserMenu;

#[component]
pub fn HeaderBar(s_get_title: ReadSignal<String>, s_get_subtitle: ReadSignal<String>) -> impl IntoView {
    let s_url = use_url();

    let is_catalog = Signal::derive(move || s_url.get().path() == "/");

    let class_name = style_sheet!("./app/src/layout/header_bar.css");

    view! { class=class_name,
        <div class="eshop-header " class:home=is_catalog>
            <div class="eshop-header-hero">
                <img
                    role="presentation"
                    src=move || {
                        if is_catalog() { "images/header-home.webp" } else { "images/header.webp" }
                    }
                />
            </div>
            <div class="eshop-header-container">
                <nav class="eshop-header-navbar">
                    <a class="logo logo-header" href="">
                        <img
                            alt="Northern Mountains"
                            src="images/logo-header.svg"
                            class="logo logo-header"
                        />
                    </a>
                    <UserMenu />
                    <CartMenu />
                </nav>
                <div class="eshop-header-intro">
                    <h1>{s_get_title}</h1>
                    <p>{s_get_subtitle}</p>
                </div>
            </div>
        </div>
    }
}
