use std::sync::Arc;

use super::product_image_url_provider_i::*;

struct ProductImageUrlProviderDefault {}

impl ProductImageUrlProvider for ProductImageUrlProviderDefault {
    fn get_product_image_url(&self, item: &CatalogItem) -> String {
        self.get_product_image_url_by_id(item.id)
    }
    fn get_product_image_url_by_id(&self, product_id: i32) -> String {
        format!("product-images/{product_id}?api-version=1.0")
    }
}

pub fn make_service() -> ProductImageUrlContext {
    ProductImageUrlContext { service: Arc::new(ProductImageUrlProviderDefault {}) }
}
