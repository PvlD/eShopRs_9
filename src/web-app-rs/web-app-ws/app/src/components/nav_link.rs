use leptos_router::location;

use super::*;

fn path_from_url(url: &mut location::Url, q_param_name: &'static str, page_index: usize, fn_index_to_string: fn(usize) -> Option<String>) -> String {
    let mut rslt = url.path().to_string();
    let search_params = url.search_params_mut();
    match fn_index_to_string(page_index) {
        Some(s) => {
            search_params.replace(q_param_name, s);
        }
        None => {
            search_params.remove(q_param_name);
        }
    }
    rslt.push_str(search_params.to_query_string().as_str());
    rslt
}

const PAGE_INDEX_MIN: usize = 1;

#[derive(Clone, Debug)]
pub struct NavLinkCb {
    pub page_index: usize,
    pub page_size: usize,
    pub count: usize,
    pub url: location::Url,
}

#[component]
pub fn NavLinkGr(sig_cb: ReadSignal<Option<NavLinkCb>>, q_param_name: &'static str, css_active_class: String, fn_index_to_string: fn(usize) -> Option<String>) -> impl IntoView {
    view! {
        {move || {
            match sig_cb() {
                None => {
                    let _: () = view! {};
                    ().into_any()
                }
                Some(cb) => {
                    match cb.count {
                        0 => {
                            let _: () = view! {};
                            ().into_any()
                        }
                        v => {
                            let class = css_active_class.clone();
                            let page_count = v / cb.page_size + 1;
                            let page_range = std::ops::RangeInclusive::new(
                                PAGE_INDEX_MIN,
                                page_count,
                            );
                            let mut url = cb.url;
                            let page_index = cb.page_index + 1;
                            let current_page_index = if page_range.contains(&page_index) {
                                page_index
                            } else {
                                *page_range.start()
                            };
                            page_range
                                .map(move |i| {
                                    let path = path_from_url(
                                        &mut url,
                                        q_param_name,
                                        i,
                                        fn_index_to_string,
                                    );
                                    if i == current_page_index {
                                        // log !("NavLinkGr sig_cb: {:?} ", sig_cb());

                                        // log!("NavLinkGr path: {}", path);
                                        view! {
                                            <a href=path class=class.clone() aria-current="page">
                                                {i}
                                            </a>
                                        }
                                            .into_any()
                                    } else {

                                        view! { <a href=path>{i}</a> }
                                            .into_any()
                                    }
                                })
                                .collect::<Vec<_>>()
                                .into_any()
                        }
                    }
                }
            }
        }}
    }
}
