use std::sync::Arc;
use tokio::sync::mpsc::UnboundedSender;

use async_trait::async_trait;

pub trait Processor<T>
where
    T: Clone + Send + Sync + 'static,
{
    fn process(&self, item: &T) -> Result<(), crate::AppError>;
}

#[derive(Clone)]
pub struct ProcessorChannel<T>
where
    T: Clone + Send + Sync + 'static,
{
    tx: UnboundedSender<T>,
    filter: Option<Arc<Box<dyn Fn(&T) -> bool + Send + Sync + 'static>>>,
}

impl<T> ProcessorChannel<T>
where
    T: Clone + Send + Sync + 'static,
{
    pub fn new(tx: UnboundedSender<T>, filter: Option<Box<dyn Fn(&T) -> bool + Send + Sync + 'static>>) -> Self {
        ProcessorChannel { tx, filter: filter.map(|f| Arc::new(f)) }
    }

    fn process(&self, item: &T) -> Result<(), crate::AppError> {
        if let Some(filter) = &self.filter {
            if filter(item) {
                self.tx.send(item.clone())?;
            }
        } else {
            self.tx.send(item.clone())?;
        }
        Ok(())
    }
}

#[async_trait]
impl<T> Processor<T> for ProcessorChannel<T>
where
    T: Clone + Send + Sync + 'static,
{
    fn process(&self, item: &T) -> Result<(), crate::AppError> {
        self.process(item)
    }
}

#[derive(Clone)]
pub struct ProcessorRedirect<T, TT>
where
    T: Clone + Send + Sync + 'static,
    TT: for<'a> From<&'a T> + Send + Sync + 'static,
{
    tx: UnboundedSender<TT>,
    filter: Option<Arc<Box<dyn Fn(&T) -> bool + Send + Sync + 'static>>>,
    _marker: std::marker::PhantomData<T>,
}

impl<T, TT> ProcessorRedirect<T, TT>
where
    T: Clone + Send + Sync + 'static,
    TT: for<'a> From<&'a T> + Send + Sync + 'static,
{
    pub fn new(tx: UnboundedSender<TT>, filter: Option<Box<dyn Fn(&T) -> bool + Send + Sync + 'static>>) -> Self {
        ProcessorRedirect {
            tx,
            filter: filter.map(|f| Arc::new(f)),
            _marker: std::marker::PhantomData,
        }
    }
}

#[async_trait]
impl<T, TT> Processor<T> for ProcessorRedirect<T, TT>
where
    T: Clone + Send + Sync + 'static,
    TT: for<'a> From<&'a T> + Send + Sync + 'static,
{
    fn process(&self, item: &T) -> Result<(), crate::AppError> {
        if let Some(filter) = &self.filter {
            if filter(item) {
                self.tx.send(TT::from(item))?;
            }
        } else {
            self.tx.send(TT::from(item))?;
        }
        Ok(())
    }
}
