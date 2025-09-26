use std::sync::Arc;

pub use catalog::types::CatalogItem;

pub trait ProductImageUrlProvider: Send + Sync {
    fn get_product_image_url(&self, item: &CatalogItem) -> String;

    fn get_product_image_url_by_id(&self, product_id: i32) -> String;
}

#[derive(Clone)]
pub struct ProductImageUrlContext {
    pub service: Arc<dyn ProductImageUrlProvider>,
}
