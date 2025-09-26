use std::{any::type_name, collections::HashMap, sync::Arc};

use crate::dispatcher::{Processor, ProcessorChannel, ProcessorRedirect};
use async_trait::async_trait;
use tokio::{
    runtime::Handle,
    sync::{
        RwLock,
        mpsc::{UnboundedReceiver, UnboundedSender},
    },
};

//use uuidmap::Table;

#[cfg(feature = "traces")]
use tracing::{error, info};

pub trait FilterFactory<T> {
    fn create(&self) -> Box<dyn Fn(&T) -> bool + Send + Sync + 'static>;
}

pub struct Dispatcher<T>
where
    T: Clone + Send + Sync + 'static,
{
    processors: Arc<RwLock<HashMap<usize, Box<dyn Processor<T> + Send + Sync + 'static>>>>,
}

pub trait Dispatcherable<T>
where
    T: Clone + Send + Sync + 'static,
{
    fn dispatcher() -> Arc<RwLock<Dispatcher<T>>>;
}

impl<T> Default for Dispatcher<T>
where
    T: Clone + Send + Sync + 'static,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Dispatcher<T>
where
    T: Clone + Send + Sync + 'static,
{
    pub fn new() -> Self {
        Self {
            processors: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}
#[async_trait]
pub trait Unsubsriber {
    async fn unsubscribe(&mut self) -> ();
}

#[derive(Clone)]
pub struct UnsubsriberForOne<T>
where
    T: Clone + Send + Sync + 'static,
{
    key: usize,
    dispatcher: Arc<RwLock<HashMap<usize, Box<dyn Processor<T> + Send + Sync + 'static>>>>,
}

impl<T> UnsubsriberForOne<T>
where
    T: Clone + Send + Sync + 'static,
{
    pub fn new(key: usize, dispatcher: Arc<RwLock<HashMap<usize, Box<dyn Processor<T> + Send + Sync + 'static>>>>) -> Self {
        Self { key, dispatcher }
    }
}

#[async_trait]
impl<T> Unsubsriber for UnsubsriberForOne<T>
where
    T: Clone + Send + Sync + 'static,
{
    async fn unsubscribe(&mut self) {
        let mut processors = self.dispatcher.write().await;
        let result = processors.remove(&self.key);
        if result.is_none() {
            #[cfg(feature = "traces")]
            error!("Unsubsriber already unsuscribed {}", type_name::<T>());
            log::error!("Unsubsriber already unsuscribed {}", type_name::<T>());
        }
        self.key = 0;
    }
}

impl<T> Drop for UnsubsriberForOne<T>
where
    T: Clone + Send + Sync + 'static,
{
    fn drop(&mut self) {
        if self.key != 0 {
            #[cfg(feature = "traces")]
            error!("Unsubsriber droppednot unsuscribed {}", type_name::<T>());

            #[cfg(feature = "traces")]
            log::error!("Unsubsriber droppednot unsuscribed {}", type_name::<T>());
            if let Ok(handle) = Handle::try_current() {
                let resource = self.key;
                let dispatcher = Arc::clone(&self.dispatcher);
                handle.spawn(async move {
                    let mut processors = dispatcher.write().await;
                    processors.remove(&resource);
                });
            } else {
                #[cfg(feature = "traces")]
                error!("No async runtime available to clean up ");
                log::error!("No async runtime available to clean up ");
            }
        }
    }
}

pub struct UnsubsriberForMany {
    data: Vec<Box<dyn Unsubsriber + Send + Sync + 'static>>,
}

impl UnsubsriberForMany {
    pub fn new(data: Vec<Box<dyn Unsubsriber + Send + Sync + 'static>>) -> Self {
        Self { data }
    }
}

#[async_trait]
impl Unsubsriber for UnsubsriberForMany {
    async fn unsubscribe(&mut self) {
        for item in self.data.iter_mut() {
            item.unsubscribe().await;
        }
        self.data.clear();
    }
}

impl Drop for UnsubsriberForMany {
    fn drop(&mut self) {
        if !self.data.is_empty() {
            #[cfg(feature = "traces")]
            error!("UnsubsriberForMany droppednot unsuscribed ");
            log::error!("UnsubsriberForMany droppednot unsuscribed ");

            while let Some(mut item) = self.data.pop() {
                if let Ok(handle) = Handle::try_current() {
                    handle.spawn(async move {
                        item.unsubscribe().await;
                    });
                } else {
                    #[cfg(feature = "traces")]
                    error!("No async runtime available to clean up ");
                    log::error!("No async runtime available to clean up ");
                }
            }
        }
    }
}

impl<T> Dispatchable<T> for Dispatcher<T>
where
    T: Clone + Send + Sync + 'static,
{
    async fn dispatch(&self, v: T) -> Result<(), crate::AppError> {
        let processors = self.processors.read().await;

        #[cfg(feature = "traces")]
        info!("dispatch processors count {} for {}", processors.len(), type_name::<T>());

        for processor in processors.iter() {
            processor.1.process(&v)?;
        }
        Ok(())
    }

    async fn processor_count(&self) -> usize {
        self.processors.read().await.len()
    }
}

pub trait Dispatchable<T>
where
    T: Clone + Send + Sync + 'static,
{
    fn dispatch(&self, v: T) -> impl std::future::Future<Output = Result<(), crate::AppError>> + Send;
    //fn add_processor(& self, processor: Box<dyn Processor<T>+ Send + Sync + 'static>) -> impl std::future::Future<Output = Box<dyn Unsubsriber + Send + Sync + 'static>> + Send;
    fn processor_count(&self) -> impl std::future::Future<Output = usize> + Send;
}

impl<T> Dispatcher<T>
where
    T: Clone + Send + Sync + 'static,
    Dispatcher<T>: Dispatchable<T>,
{
    async fn add_processor(&self, processor: Box<dyn Processor<T> + Send + Sync + 'static>) -> Box<dyn Unsubsriber + Send + Sync + 'static> {
        let key = (Box::as_ptr(&processor) as *const ()) as usize;
        let mut processors = self.processors.write().await;
        let _ = processors.insert(key, processor);
        Box::new(UnsubsriberForOne { key, dispatcher: self.processors.clone() })
    }

    pub async fn add_channel(&self, filter: Option<Box<dyn Fn(&T) -> bool + Send + Sync + 'static>>) -> (UnboundedReceiver<T>, Box<dyn Unsubsriber + Send + Sync + 'static>) {
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
        let processor = ProcessorChannel::new(tx, filter);

        let unsuscriber = self.add_processor(Box::new(processor)).await;

        #[cfg(feature = "traces")]
        info!(" add_channel channels count {} for {}", self.processors.read().await.len(), type_name::<T>());
        (rx, unsuscriber)
    }
}
impl<T> Dispatcher<T>
where
    T: Clone + Send + Sync + 'static,
    Dispatcher<T>: Dispatchable<T>,
{
    pub async fn add_channel_redirect<TT>(&mut self, tx: UnboundedSender<TT>, filter: Option<Box<dyn Fn(&T) -> bool + Send + Sync + 'static>>) -> Box<dyn Unsubsriber + Send + Sync + 'static>
    where
        TT: for<'a> From<&'a T> + Send + Sync + 'static,
    {
        let processor = ProcessorRedirect::new(tx, filter);

        let unsuscriber = self.add_processor(Box::new(processor)).await;

        #[cfg(feature = "traces")]
        info!("  add_channel_redirect channels count {} for {}", self.processors.read().await.len(), type_name::<T>());
        unsuscriber
    }
}
