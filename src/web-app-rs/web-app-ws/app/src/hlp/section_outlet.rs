use core::panic;
use leptos::prelude::*;
use leptos::reactive::signal::{ReadSignal, WriteSignal};
use std::sync::Arc;
use std::{collections::HashMap, sync::RwLock};
use wasm_bindgen::UnwrapThrowExt;

#[allow(dead_code)]
#[derive(Debug)]
struct Data {
    key: String,
    s_get: ReadSignal<String>,
    s_set: WriteSignal<String>,
}

#[derive(Clone, Debug)]
pub struct KeyDispatcher {
    data: Arc<RwLock<HashMap<String, Data>>>,
}

impl Default for KeyDispatcher {
    fn default() -> Self {
        Self::new()
    }
}

impl KeyDispatcher {
    pub fn new() -> Self {
        Self { data: Arc::new(RwLock::new(HashMap::new())) }
    }

    pub fn provide_context(&self) {
        if use_context::<KeyDispatcher>().is_some() {
            panic!("KeyDispatcher is already in context. ");
        }
        provide_context(self.clone());
    }

    pub fn register(&self, key: &str) -> ReadSignal<String> {
        let mut w = self.data.write().unwrap();
        if w.contains_key(key) {
            panic!("KeyDispatcher key: {key} already registered");
        }

        let (s_get, s_set) = signal(" ".to_string());

        let key = key.to_string();
        w.insert(key.to_owned(), Data { key, s_get, s_set });
        s_get
    }

    pub fn set_value(key: &str, value: &str) {
        let that = use_context::<KeyDispatcher>().expect_throw("KeyDispatcher not in context. ");

        let mut w = that.data.write().unwrap();

        match w.get_mut(key) {
            Some(v) => {
                v.s_set.set(value.to_string());
            }
            None => panic!("KeyDispatcher key: {key} not registered"),
        }
    }
}
