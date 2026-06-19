use crate::layout::HeaderContext;
use crate::services::*;
use leptos::prelude::*;
use leptos_router::hooks::use_query_map;

const PAGE_SIZE: i32 = 9;

#[component]
pub fn CatalogPage() -> impl IntoView {
    if let Some(ctx) = use_context::<HeaderContext>() {
        ctx.title.set("Catalog".to_string());
        ctx.subtitle.set("Items".to_string());
    }

    let query = use_query_map();
    let page = move || {
        query
            .get()
            .get("page")
            .and_then(|v| v.parse::<i32>().ok())
            .unwrap_or(1)
    };
    let brand_id = move || {
        query
            .get()
            .get("brand")
            .and_then(|v| v.parse::<i32>().ok())
    };
    let type_id = move || {
        query
            .get()
            .get("type")
            .and_then(|v| v.parse::<i32>().ok())
    };

    let catalog = LocalResource::new(move || {
        get_catalog_items(page() - 1, PAGE_SIZE, brand_id(), type_id())
    });
    let brands = LocalResource::new(|| get_catalog_brands());
    let types = LocalResource::new(|| get_catalog_types());

    view! {
        <div class="catalog">
            <Suspense fallback=|| ()>
                {move || Suspend::new(async move {
                    let brands = brands.await;
                    let types = types.await;
                    match (&brands, &types) {
                        (Ok(brands), Ok(types)) => {
                            let filter_count = [brand_id(), type_id()].iter().filter(|x| x.is_some()).count();
                            let f_brand_id = brand_id;
                            let f_type_id = type_id;
                            view! {
                                <div class="catalog-search">
                                    <div class="catalog-search-header">
                                        <img role="presentation" src="/icons/filters.svg" alt="filters" />
                                        <span>"Filters"</span>
                                        <span class="search-badge">{filter_count}</span>
                                    </div>
                                    <div class="catalog-search-types">
                                        <div class="catalog-search-group">
                                            <h3>"Brand"</h3>
                                            <div class="catalog-search-group-tags">
                                                <a href=move || {
                                                    match f_type_id() {
                                                        Some(t) => format!("/?type={t}"),
                                                        None => "/".to_string(),
                                                    }
                                                }
                                                    class=move || if f_brand_id().is_none() { "catalog-search-tag active" } else { "catalog-search-tag" }>
                                                    "All"
                                                </a>
                                                {brands.iter().map(|b| {
                                                    let id = b.id;
                                                    let f_type_id = f_type_id;
                                                    let href = move || {
                                                        match f_type_id() {
                                                            Some(t) => format!("/?brand={id}&type={t}"),
                                                            None => format!("/?brand={id}"),
                                                        }
                                                    };
                                                    let name = b.brand.clone();
                                                    view! {
                                                        <a href=href
                                                            class=move || if f_brand_id() == Some(id) { "catalog-search-tag active" } else { "catalog-search-tag" }>
                                                            {name}
                                                        </a>
                                                    }
                                                }).collect::<Vec<_>>()}
                                            </div>
                                        </div>
                                        <div class="catalog-search-group">
                                            <h3>"Type"</h3>
                                            <div class="catalog-search-group-tags">
                                                <a href=move || {
                                                    match f_brand_id() {
                                                        Some(b) => format!("/?brand={b}"),
                                                        None => "/".to_string(),
                                                    }
                                                }
                                                    class=move || if f_type_id().is_none() { "catalog-search-tag active" } else { "catalog-search-tag" }>
                                                    "All"
                                                </a>
                                                {types.iter().map(|t| {
                                                    let id = t.id;
                                                    let f_brand_id = f_brand_id;
                                                    let href = move || {
                                                        match f_brand_id() {
                                                            Some(b) => format!("/?type={id}&brand={b}"),
                                                            None => format!("/?type={id}"),
                                                        }
                                                    };
                                                    let name = t.name.clone();
                                                    view! {
                                                        <a href=href
                                                            class=move || if f_type_id() == Some(id) { "catalog-search-tag active" } else { "catalog-search-tag" }>
                                                            {name}
                                                        </a>
                                                    }
                                                }).collect::<Vec<_>>()}
                                            </div>
                                        </div>
                                    </div>
                                </div>
                            }.into_any()
                        }
                        _ => view! { <p>"Loading filters..."</p> }.into_any(),
                    }
                })}
            </Suspense>

            <div class="catalog-main">
                <Suspense fallback=|| view! { <p>"Loading..."</p> }>
                    {move || Suspend::new(async move {
                        let result = catalog.await;
                        match result.as_ref() {
                            Ok(catalog_result) => {
                                let total_pages = (catalog_result.count as f64 / PAGE_SIZE as f64).ceil() as i32;
                                let current_page = page();
                                view! {
                                    <div class="catalog-items">
                                        {catalog_result.data.iter().map(|item| {
                                            let img_src = format!("/product-images/{}?api-version=2.0", item.id);
                                            let name = item.name.clone();
                                            let price = format!("${:.2}", item.price);
                                            let href = format!("/item/{}", item.id);
                                            view! {
                                                <div class="catalog-item">
                                                    <a href=href class="catalog-product">
                                                        <span class="catalog-product-image">
                                                            <img alt=name.clone() src=img_src />
                                                        </span>
                                                        <span class="catalog-product-content">
                                                            <span class="name">{name}</span>
                                                            <span class="price">{price}</span>
                                                        </span>
                                                    </a>
                                                </div>
                                            }
                                        }).collect::<Vec<_>>()}
                                    </div>
                                    <div class="page-links">
                                        {(1..=total_pages).map(|page_index| {
                                            let is_active = page_index == current_page;
                                            let href = if page_index == 1 {
                                                "/".to_string()
                                            } else {
                                                format!("/?page={}", page_index)
                                            };
                                            view! {
                                                <a href=href class=move || { if is_active { "active" } else { "" } }>
                                                    {page_index}
                                                </a>
                                            }
                                        }).collect::<Vec<_>>()}
                                    </div>
                                }.into_any()
                            }
                            Err(e) => view! { <p>"Error: " {e.to_string()}</p> }.into_any(),
                        }
                    })}
                </Suspense>
            </div>
        </div>
    }
}
