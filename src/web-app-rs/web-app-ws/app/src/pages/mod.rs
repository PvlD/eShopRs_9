mod cart;
mod catalog;
mod checkout;
mod item;
mod orders;
pub(crate) use cart::CartPage;
pub(crate) use catalog::CatalogPage;
pub(crate) use checkout::CheckoutPage;
pub(crate) use item::ItemPage;
pub(crate) use orders::OrdersPage;

mod parameter_from_query {

    use leptos::Params;
    use leptos_router::params::Params;

    #[derive(Params, PartialEq)]
    pub struct Page {
        pub page: Option<usize>,
    }

    #[derive(Params, PartialEq)]
    pub struct BrandId {
        pub brand: Option<usize>,
    }
}
