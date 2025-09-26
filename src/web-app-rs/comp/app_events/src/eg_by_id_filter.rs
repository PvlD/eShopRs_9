use rabbit_mq_bus::Dispatcherable;
use rabbit_mq_bus::Unsubsriber;
use rabbit_mq_bus::UnsubsriberForMany;
use tokio::sync::mpsc::UnboundedReceiver;
use tokio::sync::mpsc::UnboundedSender;

use crate::integration_events::OrderStatusChangedToAwaitingValidation;
use crate::integration_events::OrderStatusChangedToCancelled;
use crate::integration_events::OrderStatusChangedToPaid;
use crate::integration_events::OrderStatusChangedToShipped;
use crate::integration_events::OrderStatusChangedToStockConfirmed;
use crate::integration_events::OrderStatusChangedToSubmitted;

use rabbit_mq_bus::FilterFactory;

pub trait BuyerIdentity {
    fn buyer_identity(&self) -> String;
}

pub struct EvRedirect {
    pub order_id: i32,
    pub order_status: String,
}

impl From<&OrderStatusChangedToStockConfirmed> for EvRedirect {
    fn from(value: &OrderStatusChangedToStockConfirmed) -> Self {
        EvRedirect {
            order_id: value.order_id,
            order_status: value.order_status.clone(),
        }
    }
}

impl BuyerIdentity for OrderStatusChangedToStockConfirmed {
    fn buyer_identity(&self) -> String {
        self.buyer_identity_guid.clone()
    }
}

impl From<&OrderStatusChangedToAwaitingValidation> for EvRedirect {
    fn from(value: &OrderStatusChangedToAwaitingValidation) -> Self {
        EvRedirect {
            order_id: value.order_id,
            order_status: value.order_status.clone(),
        }
    }
}

impl BuyerIdentity for OrderStatusChangedToAwaitingValidation {
    fn buyer_identity(&self) -> String {
        self.buyer_identity_guid.clone()
    }
}

impl From<&OrderStatusChangedToCancelled> for EvRedirect {
    fn from(value: &OrderStatusChangedToCancelled) -> Self {
        EvRedirect {
            order_id: value.order_id,
            order_status: value.order_status.clone(),
        }
    }
}

impl BuyerIdentity for OrderStatusChangedToCancelled {
    fn buyer_identity(&self) -> String {
        self.buyer_identity_guid.clone()
    }
}

impl From<&OrderStatusChangedToPaid> for EvRedirect {
    fn from(value: &OrderStatusChangedToPaid) -> Self {
        EvRedirect {
            order_id: value.order_id,
            order_status: value.order_status.clone(),
        }
    }
}

impl BuyerIdentity for OrderStatusChangedToPaid {
    fn buyer_identity(&self) -> String {
        self.buyer_identity_guid.clone()
    }
}

impl From<&OrderStatusChangedToShipped> for EvRedirect {
    fn from(value: &OrderStatusChangedToShipped) -> Self {
        EvRedirect {
            order_id: value.order_id,
            order_status: value.order_status.clone(),
        }
    }
}

impl BuyerIdentity for OrderStatusChangedToShipped {
    fn buyer_identity(&self) -> String {
        self.buyer_identity_guid.clone()
    }
}

impl From<&OrderStatusChangedToSubmitted> for EvRedirect {
    fn from(value: &OrderStatusChangedToSubmitted) -> Self {
        EvRedirect {
            order_id: value.order_id,
            order_status: value.order_status.clone(),
        }
    }
}

impl BuyerIdentity for OrderStatusChangedToSubmitted {
    fn buyer_identity(&self) -> String {
        self.buyer_identity_guid.clone()
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

async fn add_channel_redirect<TT, T>(tx: UnboundedSender<TT>, filter_factory: &Option<&FilterFactoryByID>) -> Box<dyn Unsubsriber + Send + Sync + 'static>
where
    T: Dispatcherable<T> + BuyerIdentity + Clone + Send + Sync + 'static,
    TT: for<'a> From<&'a T> + Send + Sync + 'static,
{
    (T::dispatcher().write().await.add_channel_redirect(tx.clone(), filter_factory.as_ref().map(|f| f.create())).await) as _
}

pub async fn register_group(buyer_identity: Option<String>) -> (UnboundedReceiver<EvRedirect>, Box<dyn Unsubsriber + Send + Sync + 'static>) {
    let filter_factory: Option<FilterFactoryByID> = if buyer_identity.is_some() { Some(FilterFactoryByID::new(buyer_identity.unwrap())) } else { None };
    let filter_factory: Option<&FilterFactoryByID> = filter_factory.as_ref();

    let (tx, rx) = tokio::sync::mpsc::unbounded_channel::<EvRedirect>();

    let unsuscriber = UnsubsriberForMany::new(vec![
        add_channel_redirect::<EvRedirect, OrderStatusChangedToStockConfirmed>(tx.clone(), &filter_factory).await,
        add_channel_redirect::<EvRedirect, OrderStatusChangedToCancelled>(tx.clone(), &filter_factory).await,
        add_channel_redirect::<EvRedirect, OrderStatusChangedToPaid>(tx.clone(), &filter_factory).await,
        add_channel_redirect::<EvRedirect, OrderStatusChangedToShipped>(tx.clone(), &filter_factory).await,
        add_channel_redirect::<EvRedirect, OrderStatusChangedToSubmitted>(tx.clone(), &filter_factory).await,
        add_channel_redirect::<EvRedirect, OrderStatusChangedToAwaitingValidation>(tx.clone(), &filter_factory).await,
    ]);

    (rx, Box::new(unsuscriber))
}
