use std::sync::Arc;

use rabbit_mq_bus::ContentProcessor;

use crate::integration_events::OrderStatusChangedToAwaitingValidation;
use crate::integration_events::OrderStatusChangedToCancelled;
use crate::integration_events::OrderStatusChangedToPaid;
use crate::integration_events::OrderStatusChangedToShipped;
use crate::integration_events::OrderStatusChangedToStockConfirmed;
use crate::integration_events::OrderStatusChangedToSubmitted;

pub fn register(processor: &mut Arc<ContentProcessor>) {
    let processor = Arc::get_mut(processor).unwrap();
    processor.register::<OrderStatusChangedToStockConfirmed>();
    processor.register::<OrderStatusChangedToAwaitingValidation>();
    processor.register::<OrderStatusChangedToCancelled>();
    processor.register::<OrderStatusChangedToPaid>();
    processor.register::<OrderStatusChangedToShipped>();
    processor.register::<OrderStatusChangedToSubmitted>();
}
