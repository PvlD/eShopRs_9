use leptos::prelude::*;

use stylers::style_sheet;

#[component]
pub fn FooterBar() -> impl IntoView {
    let class_name = style_sheet!("./app/src/layout/footer_bar.css");

    view! { class=class_name,
        <footer class="eshop-footer">
            <div class="eshop-footer-content">
                <div class="eshop-footer-row">
                    <img
                        role="presentation"
                        src="images/logo-footer.svg"
                        class="logo logo-footer"
                    />
                    <p>"© AdventureWorks"</p>
                </div>
            </div>
        </footer>
    }
}
