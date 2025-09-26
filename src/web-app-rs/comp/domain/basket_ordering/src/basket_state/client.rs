use leptos::prelude::*;

use crate::basket_state::{self, types::*};

#[derive(Clone, Debug)]
pub struct BasketStateInfo {
    pub basket_items: Result<Option<Vec<BasketItem>>, crate::AppError>,
}

impl BasketStateInfo {
    pub fn new() -> Self {
        Self { basket_items: Ok(None) }
    }

    pub fn basket_item_by_id<'a>(self: &'a BasketStateInfo, id: i32) -> Result<Option<&'a BasketItem>, crate::AppError> {
        match self.basket_items.as_ref() {
            Ok(Some(basket_items)) => Ok(basket_items.iter().find(|item| item.product_id == id)),
            Ok(None) => Ok(None),
            Err(e) => Err(e.clone()),
        }
    }
}

#[derive(Clone)]
pub struct BasketStateInfoContext(pub RwSignal<BasketStateInfo>);

impl BasketStateInfoContext {
    pub fn new() -> Self {
        Self(RwSignal::new(BasketStateInfo::new()))
    }

    pub fn update_basket_state_info(&self, data: BasketStateInfo) {
        self.0.update(|basket_state_info| *basket_state_info = data);
    }
}

pub fn signal_from_context() -> RwSignal<BasketStateInfo> {
    let context = expect_context::<BasketStateInfoContext>();
    context.0
}

pub fn provide_basket_state_info_context() {
    provide_context(BasketStateInfoContext::new());
}

pub async fn refresh_basket_state_info(context: BasketStateInfoContext, auth_context: auth::client::UserInfoCntxt) {
    if !auth_context.is_logged_in() {
        context.0.update(|basket_state_info| {
            basket_state_info.basket_items = Ok(None);
        });
    } else {
        let basket_items = basket_state::server_api::get_basket_items().await;
        let basket_items = basket_items.map(|basket_items| Some(basket_items));
        context.0.update(|basket_state_info| {
            basket_state_info.basket_items = basket_items;
        });
    }
}

pub fn refresh_basket_state_info_action() -> Action<(), ()> {
    Action::new(move |_: &()| {
        let context = expect_context::<BasketStateInfoContext>();
        let auth_context = expect_context::<auth::client::UserInfoCntxt>();
        refresh_basket_state_info(context, auth_context)
    })
}
