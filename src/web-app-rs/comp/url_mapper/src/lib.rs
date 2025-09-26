mod internal;

use std::sync::Arc;

pub trait UrlMap: Send + Sync {
    fn get_mapped_url(&self, key: &str) -> Option<&str>;
    fn to_string(&self) -> String;
}

use core::fmt::Debug;
impl Debug for dyn UrlMap {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "UrlMap{{{}}}", self.to_string())
    }
}

pub type UrlMapService = Arc<dyn UrlMap>;

pub fn from_env() -> UrlMapService {
    Arc::new(internal::InternalImpl::new_from_env())
}

pub fn from_vec(vec: &Vec<(&str, &str)>) -> UrlMapService {
    Arc::new(internal::InternalImpl::new_from_vec(vec))
}
