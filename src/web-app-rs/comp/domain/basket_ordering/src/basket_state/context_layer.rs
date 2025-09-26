use axum::{body::Body, http, response::Response};
use http::Request;

use leptos::prelude::provide_context;
use pin_project_lite::pin_project;
use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};
use tower::{Layer, Service};

pub struct BasketStateServiceLayer;

impl<S> Layer<S> for BasketStateServiceLayer {
    type Service = BasketStateServiceLayerService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        BasketStateServiceLayerService { inner }
    }
}

pub struct BasketStateServiceLayerService<T> {
    inner: T,
}

impl<T> Service<Request<Body>> for BasketStateServiceLayerService<T>
where
    T: Service<Request<Body>, Response = Response<Body>> + Send + 'static,
    T::Future: Send + 'static,
{
    type Response = T::Response;
    type Error = T::Error;
    type Future = BasketStateServiceLayerServiceFuture<T>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<Body>) -> Self::Future {
        provide_context(crate::basket_state::server::make_service_from_context().unwrap());

        BasketStateServiceLayerServiceFuture { inner: self.inner.call(req) }
    }
}

pin_project! {
    pub struct BasketStateServiceLayerServiceFuture<S>
    where
        S: Service<Request<Body>, Response = Response<Body>>,
    {
        #[pin]
        inner: S::Future,
    }
}

impl<S> Future for BasketStateServiceLayerServiceFuture<S>
where
    S: Service<Request<Body>, Response = Response<Body>>,
{
    type Output = Result<S::Response, S::Error>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        this.inner.poll(cx)
    }
}
