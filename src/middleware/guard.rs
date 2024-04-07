//! Request guard for protecting a router against non-htmx requests.

use std::{
    fmt,
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

use futures_core::ready;
use http::{header::LOCATION, response::Response, Request, StatusCode};
use pin_project_lite::pin_project;
use tower::{Layer, Service};

use crate::HX_REQUEST;

/// Checks if the request contains the `HX-Request` header, redirecting to the
/// given location if not.
///
/// This can be useful for preventing users from accidently ending up on a route
/// which would otherwise return only partial HTML data.
#[derive(Debug, Clone)]
pub struct HxRequestGuardLayer<'a> {
    redirect_to: &'a str,
}

impl<'a> HxRequestGuardLayer<'a> {
    #[inline]
    pub fn new(redirect_to: &'a str) -> Self {
        Self { redirect_to }
    }
}

impl Default for HxRequestGuardLayer<'_> {
    #[inline]
    fn default() -> Self {
        Self { redirect_to: "/" }
    }
}

impl<'a, S> Layer<S> for HxRequestGuardLayer<'a> {
    type Service = HxRequestGuard<'a, S>;

    #[inline]
    fn layer(&self, inner: S) -> Self::Service {
        HxRequestGuard {
            inner,
            layer: self.clone(),
        }
    }
}

/// Tower service that implementes redirecting to non-partial routes.
#[derive(Debug, Clone)]
pub struct HxRequestGuard<'a, S> {
    inner: S,
    layer: HxRequestGuardLayer<'a>,
}

impl<'a, S, T, U> Service<Request<T>> for HxRequestGuard<'a, S>
where
    S: Service<Request<T>, Response = Response<U>>,
    U: Default,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = private::ResponseFuture<'a, S::Future>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<T>) -> Self::Future {
        // This will always contain a "true" value.
        let hx_request = req.headers().contains_key(HX_REQUEST);
        let response_future = self.inner.call(req);

        private::ResponseFuture {
            response_future,
            hx_request,
            layer: self.layer.clone(),
        }
    }
}

mod private {
    use super::*;

    pin_project! {
        pub struct ResponseFuture<'a, F> {
            #[pin]
            pub(super) response_future: F,
            pub(super) hx_request: bool,
            pub(super) layer: HxRequestGuardLayer<'a>,
        }
    }

    impl<'a, F, B, E> Future for ResponseFuture<'a, F>
    where
        F: Future<Output = Result<Response<B>, E>>,
        B: Default,
    {
        type Output = Result<Response<B>, E>;

        fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
            let this = self.project();
            let response: Response<B> = ready!(this.response_future.poll(cx))?;

            match *this.hx_request {
                true => Poll::Ready(Ok(response)),
                false => {
                    let res = Response::builder()
                        .status(StatusCode::SEE_OTHER)
                        .header(LOCATION, this.layer.redirect_to)
                        .body(B::default())
                        .expect("failed to build response");

                    Poll::Ready(Ok(res))
                }
            }
        }
    }
}

#[derive(Debug, Default)]
struct HxRequestGuardError;

impl fmt::Display for HxRequestGuardError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("HxRequestGuardError")
    }
}

impl std::error::Error for HxRequestGuardError {}
