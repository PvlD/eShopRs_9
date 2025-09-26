#[cfg(test)]
#[path = "../dispatcher.rs"]
mod dispatcher;

use dispatcher::{Dispatchable, Dispatcher};

use crate::events::ev_1::Ev1;
use crate::{eg_by_id_filter::EvRedirect, trace_init};

#[tokio::test]
async fn dispatcher() {
    let mut dispatcher = Dispatcher::<Ev1>::new();

    let (mut rx, mut unsuscriber) = dispatcher.add_channel(None).await;
    let ev1 = Ev1 {
        data: "test".to_string(),
        buyer_identity_guid: "test".to_string(),
    };
    let dispatcher = dispatcher;
    let _result = dispatcher.dispatch(ev1).await;

    let result = rx.recv().await;
    unsuscriber.unsubscribe().await;
    assert!(result.is_some());
}

#[tokio::test]
async fn dispatcher_with_unsubsriber() {
    let mut dispatcher = Dispatcher::<Ev1>::new();
    {
        let (mut rx, mut unsuscriber) = dispatcher.add_channel(None).await;
        let ev1 = Ev1 {
            data: "test".to_string(),
            buyer_identity_guid: "test".to_string(),
        };
        let dispatcher = dispatcher;
        let _result = dispatcher.dispatch(ev1).await;

        let result = rx.recv().await;
        unsuscriber.unsubscribe().await;
        assert!(result.is_some());

        let count = dispatcher.processor_count().await;
        assert_eq!(count, 0);
    }
}

#[tokio::test]
async fn dispatcher_with_unsubsriber_forgot_unsuscribe() {
    let mut dispatcher = Dispatcher::<Ev1>::new();
    {
        let (mut rx, unsuscriber) = dispatcher.add_channel(None).await;
        let ev1 = Ev1 {
            data: "test".to_string(),
            buyer_identity_guid: "test".to_string(),
        };

        let _result = dispatcher.dispatch(ev1).await;

        let result = rx.recv().await;
        //unsuscriber.unsuscribe().await;
        assert!(result.is_some());
    }
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    let count = dispatcher.processor_count().await;
    assert_eq!(count, 0, "processor count should be 0");
}

#[tokio::test]
async fn dispatcher_with_unsubsriber_and_redirect() {
    trace_init();

    let mut dispatcher = Dispatcher::<Ev1>::new();
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<EvRedirect>();

    let mut unsuscriber = dispatcher.add_channel_redirect(tx, None).await;
    let ev1 = Ev1 {
        data: "test".to_string(),
        buyer_identity_guid: "test".to_string(),
    };
    let dispatcher = dispatcher;
    let _result = dispatcher.dispatch(ev1).await;

    let result: Option<EvRedirect> = rx.recv().await;
    unsuscriber.unsubscribe().await;
    assert!(result.is_some());
    let count = dispatcher.processor_count().await;
    assert_eq!(count, 0);
}
