use leptos::prelude::*;
//use leptos::*;

use leptos_meta::Title;
use leptos_router::hooks::{self, use_query, use_query_map};

use stylers::style_sheet;

use crate::{
    components::{NavLinkCb, NavLinkGr},
    pages::{
        catalog::{CatalogPageError, CatalogSearch},
        parameter_from_query,
    },
};

use error_template::ErrorTemplate;

use catalog::server_api::get_catalog_items;

const PAGE_SIZE: usize = 9;

#[component]
pub fn CatalogPage() -> impl IntoView {
    let sig_url: ReadSignal<leptos_router::location::Url> = hooks::use_url();

    let page = use_query::<parameter_from_query::Page>();
    let sig_page_index = Signal::derive(move || page.with(|page| page.as_ref().map(|d| d.page).map_err(|_| CatalogPageError::InvalidPageIndex)));

    let brand = use_query::<parameter_from_query::BrandId>();
    let sig_brand_id = Signal::derive(move || brand.with(|d| d.as_ref().map(|d| d.brand).map_err(|_| CatalogPageError::InvalidBrandId)));

    let params = use_query_map();

    let sig_type_id = Signal::derive(move || {
        params()
            //``.read()
            .get("type")
            .map(|d| d.parse::<usize>())
            .transpose()
            .map_err(|_| CatalogPageError::InvalidItemTypeId)
    });

    let catalog = Resource::new(
        move || (sig_page_index(), sig_brand_id(), sig_type_id()),
        |(page, brand, typeid)| async move {
            //log!("Resource Catalog page: {:?} brand: {:?} typeid: {:?}", page, brand, typeid);
            let page = page?.map(|p| if p == 0 { p } else { p - 1 }).unwrap_or(0);
            get_catalog_items(page, PAGE_SIZE, brand?, typeid?).await
        },
    );

    let sig_cb: RwSignal<Option<NavLinkCb>> = RwSignal::new(None);

    let class_name_item = style_sheet!("./app/src/pages/catalog/catalog_list_item.css");

    let items_view = move || {
        Suspend::new(async move {
            //log!("items_view class_name_item: {}", class_name_item);

            view! { class=class_name_item,
                {catalog
                    .await
                    .map(|catalog| {
                        sig_cb
                            .set(
                                Some(NavLinkCb {
                                    url: sig_url.get_untracked(),
                                    page_index: catalog.page_index,
                                    page_size: catalog.page_size,
                                    count: catalog.count,
                                }),
                            );
                        catalog
                            .data
                            .into_iter()
                            .map(|item| {
                                // log!("items_view catalog: {:?}", catalog);
                                view! {
                                    <super::catalog_list_item::CatalogListItem
                                        class_name=class_name_item
                                        item=item
                                    />
                                }
                                    .into_view()
                            })
                            .collect::<Vec<_>>()
                    })}
            }
        })
    };

    let class_name = style_sheet!("./app/src/pages/catalog/catalog.css");

    let nav_link_view = move || {
        Suspend::new(async move {
            view! { class=class_name,
                <div class="page-links">

                    <NavLinkGr
                        css_active_class="active-page".to_string()
                        sig_cb=sig_cb.read_only()
                        q_param_name="page"
                        fn_index_to_string=|i| if i == 1 { None } else { Some(i.to_string()) }
                    />
                </div>
            }
            .into_any()
        })
    };

    crate::app::page_header::set_title("Ready for a new adventure?");
    crate::app::page_header::set_subtitle("Start the season with the latest in clothing and equipment.");

    view! { class=class_name,
        <Title text="Northern Mountains" />
        <Suspense fallback=move || view! { <p>"Loading data..."</p> }>
            <ErrorBoundary fallback=|errors| view! { <ErrorTemplate errors /> }>

                <div class="catalog">

                    <CatalogSearch sig_brand_id sig_type_id />
                    <div>

                        <div class="catalog-items">{items_view}</div>
                        {nav_link_view}

                    </div>
                </div>
            </ErrorBoundary>
        </Suspense>
    }
}
