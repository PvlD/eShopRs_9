use tokio::sync::mpsc::error::TryRecvError;

use crate::Ev1;
use crate::Ev2;
use crate::eg_by_id_filter;
use crate::trace_init;
use ebus::{Dispatchable, Dispatcherable};

#[tokio::test]
async fn tst_group_by_buyer_identity() {
    trace_init();

    let (mut rx, mut unsuscriber) = eg_by_id_filter::register_group("test".to_string()).await;

    let d1 = Ev1::dispatcher();
    let d2 = Ev2::dispatcher();

    let e1 = Ev1 {
        data: "data1".to_string(),
        buyer_identity_guid: "test".to_string(),
    };
    let e2 = Ev2 {
        data: "data2".to_string(),
        buyer_identity_guid: "test".to_string(),
    };

    d1.write().await.dispatch(e1).await;
    d2.write().await.dispatch(e2).await;

    let result = rx.recv().await;
    assert!(result.is_some());

    let result = rx.recv().await;
    assert!(result.is_some());

    unsuscriber.unsubscribe().await;

    let count = d1.read().await.processor_count().await;
    assert_eq!(count, 0);
    let count = d2.read().await.processor_count().await;
    assert_eq!(count, 0);
}

#[tokio::test]
async fn tst_group_by_buyer_identity_filter() {
    trace_init();
    let (mut rx, mut unsuscriber) = eg_by_id_filter::register_group("test".to_string()).await;

    let d1 = Ev1::dispatcher();
    let d2 = Ev2::dispatcher();

    let e1 = Ev1 {
        data: "data1".to_string(),
        buyer_identity_guid: "testZ".to_string(),
    };
    let e2 = Ev2 {
        data: "data2".to_string(),
        buyer_identity_guid: "testZ".to_string(),
    };

    let _ = d1.write().await.dispatch(e1).await;
    let _ = d2.write().await.dispatch(e2).await;

    let result = rx.try_recv();
    assert!(result.err().unwrap() == TryRecvError::Empty);

    unsuscriber.unsubscribe().await;

    let count = d1.read().await.processor_count().await;
    assert_eq!(count, 0);
    let count = d2.read().await.processor_count().await;
    assert_eq!(count, 0);
}
