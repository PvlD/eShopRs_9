use axum::{body::Body, http, response::Response};
use http::Request;
use leptos::server_fn::response::Res;
use pin_project_lite::pin_project;
use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};
use tower::{Layer, Service};

pub struct RequireAuth;

impl<S> Layer<S> for RequireAuth {
    type Service = RequireAuthService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        RequireAuthService { inner }
    }
}

pub struct RequireAuthService<T> {
    inner: T,
}

impl<T> Service<Request<Body>> for RequireAuthService<T>
where
    T: Service<Request<Body>, Response = Response<Body>> + Send + 'static,
    T::Future: Send + 'static,
{
    type Response = T::Response;
    type Error = T::Error;
    type Future = AuthorisedServiceFuture<T>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<Body>) -> Self::Future {
        //println!("1. Running my middleware!");

        let is_authenticated = crate::server::is_authenticated_from_extensions(req.extensions());
        //leptos::logging::log!("is_authenticated: {:?}", is_authenticated);

        match is_authenticated {
            Ok(true) => AuthorisedServiceFuture {
                inner: self.inner.call(req),
                err_response: None,
            },

            Ok(false) => {
                let path = req.uri().path().to_string();

                let data = serde_json::to_string(&app_err::AppError::Unauthorized).unwrap();
                let res = http::Response::<axum::body::Body>::error_response(&path, data.into());
                AuthorisedServiceFuture {
                    inner: self.inner.call(req),
                    err_response: Some(res),
                }
            }
            Err(e) => {
                let path = req.uri().path().to_string();
                let data = serde_json::to_string(&app_err::AppError::ServerFnError(e)).unwrap();
                let res = http::Response::<axum::body::Body>::error_response(&path, data.into());
                AuthorisedServiceFuture {
                    inner: self.inner.call(req),
                    err_response: Some(res),
                }
            }
        }
    }
}

pin_project! {
    pub struct AuthorisedServiceFuture<S>
    where
        S: Service<Request<Body>, Response = Response<Body>>,
    {
        #[pin]
        inner: S::Future,
        err_response: Option<Response<Body>>,
    }
}

impl<S> Future for AuthorisedServiceFuture<S>
where
    S: Service<Request<Body>, Response = Response<Body>>,
{
    type Output = Result<S::Response, S::Error>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.err_response.is_some() {
            let this = self.project();
            let err_response = (*this.err_response).take();
            let err_response = err_response.unwrap();
            return Poll::Ready(Ok(err_response));
        }

        let this = self.project();
        this.inner.poll(cx)
    }
}
