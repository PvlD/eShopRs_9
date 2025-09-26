use leptos::prelude::*;

use crate::services::product_image_url_provider::ProductImageUrlContext;
use catalog::types::CatalogItem;

#[component]
pub(super) fn CatalogListItem(class_name: &'static str, item: CatalogItem) -> impl IntoView {
    let product_image_url_context = expect_context::<ProductImageUrlContext>();
    fn item_url(item: &CatalogItem) -> String {
        format!("item/{}", item.id)
    }

    let price = format!("{:.2}", (item.price));

    view! { class=class_name,
        <div class="catalog-item">
            <a class="catalog-product" href=item_url(&item)>

                <span class="catalog-product-image">
                    <img
                        alt=item.name.clone()
                        src=product_image_url_context.service.get_product_image_url(&item)
                    />
                </span>
                <span class="catalog-product-content">
                    <span class="name">{item.name.clone()}</span>
                    <span class="price">{price}</span>
                </span>
            </a>
        </div>
    }
}
