use crate::content;
use crate::content::content::EventContentProcessorDispatcher;

use crate::Dispatcherable;
use crate::content::content::ContentProcessor;
use crate::content::processor::{Content, EventContentProcessor, KeyedContentProcessor};
use crate::eg_by_id_filter::EvRedirect;
use crate::events::ev_1::Ev1;

#[tokio::test]
async fn content_processor() {
    let mut content_processor = ContentProcessor::new();

    let mut rx = Ev1::dispatcher().write().await.add_channel(None).await;

    content_processor.register::<Ev1>();

    let content = Ev1 {
        data: "test".to_string(),
        buyer_identity_guid: "test".to_string(),
    };
    let (key, content) = content.content().unwrap();

    let _result = content_processor.process(key, content).await;

    let result = rx.0.recv().await;
    assert!(result.is_some());
}

#[tokio::test]
async fn content_processor_with_redirect() {
    let mut content_processor = ContentProcessor::new();

    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<EvRedirect>();

    let mut unsuscriber = Ev1::dispatcher().write().await.add_channel_redirect::<EvRedirect>(tx, None).await;

    content_processor.register::<Ev1>();

    let content = Ev1 {
        data: "test".to_string(),
        buyer_identity_guid: "test".to_string(),
    };
    let (key, content) = content.content().unwrap();

    let _result = content_processor.process(key, content).await;

    let result = rx.recv().await;
    assert!(result.is_some());
    unsuscriber.unsubscribe().await;
}
