use leptos::prelude::*;
use leptos::*;

use leptos_router::{
    hooks::{self},
    location,
};
use logging::log;
use stylers::style_sheet;

use crate::pages::catalog::CatalogPageError;

use catalog::server_api::{get_brands, get_types};

fn path_from_url(url: &mut location::Url, params: &[(&'static str, Option<String>)]) -> String {
    let mut rslt = url.path().to_string();
    let search_params = url.search_params_mut();

    params.iter().for_each(|(k, v)| {
        if let Some(v) = v {
            search_params.replace(*k, v.to_string());
        } else {
            search_params.remove(k);
        }
    });
    rslt.push_str(search_params.to_query_string().as_str());
    rslt
}

#[component]
pub fn CatalogSearch(sig_brand_id: Signal<Result<Option<usize>, CatalogPageError>>, sig_type_id: Signal<Result<Option<usize>, CatalogPageError>>) -> impl IntoView {
    use futures::join;

    let sig_url: ReadSignal<leptos_router::location::Url> = hooks::use_url();

    let barand_item_types = Resource::new(
        move || (),
        |_| async move {
            log!("barand_item_types");
            match join!(get_brands(), get_types()) {
                (Ok(barand), Ok(item_types)) => Ok((barand, item_types)),
                (Err(e), Ok(_)) => Err(format!("brands: {:?}", e)),
                (Ok(_), Err(e)) => Err(format!("types: {:?}", e)),
                (Err(e1), Err(e2)) => Err(format!("brands: {:?}  types:{:?}", e1, e2)),
            }
        },
    );

    let barand_item_types = Memo::new(move |_| barand_item_types.get());

    #[derive(Clone)]
    struct ParamsData {
        brand_id: Option<usize>,
        item_type_id: Option<usize>,
        //errors: Option<Vec<CatalogPageError>>
    }

    let params_or_err = Signal::derive(move || {
        let brand_id = sig_brand_id();
        let item_type_id = sig_type_id();

        //log! ("params_or_err brand_id: {:?} item_type_id: {:?}",brand_id,item_type_id);

        let err = [&brand_id, &item_type_id].map(|i| if i.is_err() { i.clone().err() } else { None }).into_iter().flatten().collect::<Vec<_>>();

        if !err.is_empty() {
            Err(err)
        } else {
            Ok(ParamsData {
                brand_id: brand_id.unwrap(),
                item_type_id: item_type_id.unwrap(),
            })
        }
    });

    fn brand_uri(sig_url: ReadSignal<leptos_router::location::Url>, brand_id: Option<usize>) -> String {
        let mut url = sig_url.get();
        let params = [("page", None), ("brand", brand_id.map(|x| x.to_string()))];

        path_from_url(&mut url, &params[..])
    }

    fn type_uri(sig_url: ReadSignal<leptos_router::location::Url>, type_id: Option<usize>) -> String {
        let mut url = sig_url.get();

        let params = [("page", None), ("type", type_id.map(|x| x.to_string()))];

        path_from_url(&mut url, &params[..])
    }

    let class_name = style_sheet!("./app/src/pages/catalog/catalog_search.css");

    view! { class=class_name,
        {move || {
            barand_item_types
                .get()
                .map(|rslt| {
                    match rslt {
                        Ok((barands, item_types)) => {
                            let params = params_or_err.get();
                            match params {
                                Ok(ParamsData { brand_id, item_type_id }) => {

                                    view! { class=class_name,
                                        <div class="catalog-search">
                                            <div class="catalog-search-header">
                                                <img role="presentation" src="icons/filters.svg" />
                                                Filters
                                            </div>
                                            <div class="catalog-search-types">
                                                <div class="catalog-search-group">
                                                    <h3>Brand</h3>
                                                    <div class="catalog-search-group-tags">
                                                        <a
                                                            href=brand_uri(sig_url, None)
                                                            class="catalog-search-tag"
                                                            class:active=brand_id.is_none()
                                                        >
                                                            All
                                                        </a>
                                                        {barands
                                                            .iter()
                                                            .map(|brand| {
                                                                view! { class=class_name,
                                                                    <a
                                                                        href=brand_uri(sig_url, Some(brand.id))
                                                                        class="catalog-search-tag"
                                                                        class:active=brand_id == Some(brand.id)
                                                                    >
                                                                        {brand.brand.clone()}
                                                                    </a>
                                                                }
                                                            })
                                                            .collect::<Vec<_>>()
                                                            .into_any()}
                                                    </div>
                                                </div>
                                                <div class="catalog-search-group">
                                                    <h3>Type</h3>

                                                    <div class="catalog-search-group-tags">
                                                        <a
                                                            href=type_uri(sig_url, None)
                                                            class="catalog-search-tag"
                                                            class:active=item_type_id.is_none()
                                                        >
                                                            All
                                                        </a>
                                                        {item_types
                                                            .iter()
                                                            .map(|item_type| {
                                                                view! { class=class_name,
                                                                    <a
                                                                        href=type_uri(sig_url, Some(item_type.id))
                                                                        class="catalog-search-tag"
                                                                        class:active=item_type_id == Some(item_type.id)
                                                                    >
                                                                        {item_type.type_name.clone()}
                                                                    </a>
                                                                }
                                                            })
                                                            .collect::<Vec<_>>()
                                                            .into_any()}
                                                    </div>
                                                </div>
                                            </div>
                                        </div>
                                    }
                                        .into_any()
                                }
                                Err(ers) => {
                                    ers.into_iter()
                                        .map(|e| { Err::<String, CatalogPageError>(e.clone()) })
                                        .collect::<Vec<_>>()
                                        .into_any()
                                }
                            }
                        }
                        Err(e) => {
                            Err::<String, ServerFnError>(ServerFnError::ServerError(e)).into_any()
                        }
                    }
                })
        }}
    }
}
