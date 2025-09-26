use tokio::sync::mpsc::UnboundedReceiver;

use crate::{
    Ev1, Ev2, FilterFactory,
    dispatcher::{Dispatcherable, Unsubsriber, UnsubsriberForMany},
};

pub trait BuyerIdentity {
    fn buyer_identity(&self) -> String;
}

pub struct EvRedirect {
    pub(crate) data: String,
}

impl From<&Ev1> for EvRedirect {
    fn from(value: &Ev1) -> Self {
        EvRedirect { data: value.data.clone() }
    }
}

impl BuyerIdentity for Ev1 {
    fn buyer_identity(&self) -> String {
        self.buyer_identity_guid.clone()
    }
}

impl BuyerIdentity for Ev2 {
    fn buyer_identity(&self) -> String {
        self.buyer_identity_guid.clone()
    }
}

impl From<&Ev2> for EvRedirect {
    fn from(value: &Ev2) -> Self {
        EvRedirect { data: value.data.clone() }
    }
}

struct FilterFactoryByID {
    buyer_identity: String,
}
impl FilterFactoryByID {
    fn new(buyer_identity: String) -> Self {
        Self { buyer_identity }
    }
}

impl<T: BuyerIdentity + Send + Sync + 'static> FilterFactory<T> for FilterFactoryByID {
    fn create(&self) -> Box<dyn Fn(&T) -> bool + Send + Sync + 'static>
    where
        T: BuyerIdentity + Send + Sync + 'static,
    {
        let buyer_identity = self.buyer_identity.clone();
        Box::new(move |t: &T| t.buyer_identity() == buyer_identity)
    }
}

pub async fn register_group(buyer_identity: String) -> (UnboundedReceiver<EvRedirect>, Box<dyn Unsubsriber + Send + Sync + 'static>) {
    let filter_factory = FilterFactoryByID::new(buyer_identity);
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel::<EvRedirect>();
    let unsuscriber1 = Ev1::dispatcher().write().await.add_channel_redirect(tx.clone(), Some(filter_factory.create())).await;
    let unsuscriber2 = Ev2::dispatcher().write().await.add_channel_redirect(tx, Some(filter_factory.create())).await;
    let unsuscriber = UnsubsriberForMany::new(vec![unsuscriber1, unsuscriber2]);

    (rx, Box::new(unsuscriber))
}
