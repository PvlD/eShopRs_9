use crate::content::processor::KeyedContainer;
use crate::content::processor::KeyedContentProcessor;
use crate::dispatcher::Dispatcher;
use crate::dispatcher::Dispatcherable;
use async_trait::async_trait;
use std::collections::hash_map;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::content::processor::EventContentProcessor;
use crate::content::processor::FromContent;
use crate::content::processor::Keyed;
use crate::dispatcher::Dispatchable;

#[cfg(feature = "traces")]
use tracing::info;

pub struct EventContentProcessorDispatcher<T: FromContent + Clone + Dispatcherable<T> + Send + Sync + Keyed + 'static> {
    phantom: std::marker::PhantomData<T>,
    dispatcher: Arc<RwLock<Dispatcher<T>>>,
}

impl<T: FromContent + Clone + Dispatcherable<T> + Send + Sync + Keyed + 'static> Default for EventContentProcessorDispatcher<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: FromContent + Clone + Dispatcherable<T> + Send + Sync + Keyed + 'static> EventContentProcessorDispatcher<T> {
    pub fn new() -> Self {
        Self {
            phantom: std::marker::PhantomData,
            dispatcher: T::dispatcher(),
        }
    }

    pub fn get_dispatcher(&self) -> Arc<RwLock<Dispatcher<T>>> {
        self.dispatcher.clone()
    }
}

#[async_trait]
impl<T: FromContent + Clone + Dispatcherable<T> + Send + Sync + Keyed + 'static> EventContentProcessor for EventContentProcessorDispatcher<T> {
    async fn process(&self, content: Vec<u8>) -> Result<(), crate::AppError> {
        #[cfg(feature = "traces")]
        info!("processing event {}", T::key());

        let event = T::from_content(content)?;
        self.dispatcher.read().await.dispatch(event).await?;
        Ok(())
    }
}

pub struct ContentProcessor {
    processors: hash_map::HashMap<&'static str, Box<dyn EventContentProcessor>>,
}

impl Default for ContentProcessor {
    fn default() -> Self {
        Self::new()
    }
}

impl ContentProcessor {
    pub fn new() -> Self {
        Self { processors: hash_map::HashMap::new() }
    }

    pub fn register<T: FromContent + Clone + Dispatcherable<T> + Send + Sync + Keyed + 'static>(&mut self) -> Option<Box<dyn EventContentProcessor>> {
        self.processors.insert(T::key(), Box::new(EventContentProcessorDispatcher::<T>::new()))
    }

    pub fn get_processor(&self, key: &str) -> Option<&Box<dyn EventContentProcessor>> {
        self.processors.get(key)
    }
}

#[async_trait]
impl KeyedContainer for ContentProcessor {
    fn keys(&self) -> Vec<&'static str> {
        self.processors.keys().cloned().collect()
    }
}

#[async_trait]
impl KeyedContentProcessor for ContentProcessor {
    async fn process(&self, key: &str, content: Vec<u8>) -> Result<(), crate::AppError> {
        #[cfg(feature = "traces")]
        info!("processing content {}", key);

        self.processors
            .get(key)
            .ok_or_else(|| crate::AppError::ContentProcessorError(format!("processor not found: {}", key)))?
            .as_ref()
            .process(content)
            .await?;
        Ok(())
    }
}
