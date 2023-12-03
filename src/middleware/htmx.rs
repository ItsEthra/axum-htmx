#![allow(dead_code)]

use std::{
    future::Future,
    pin::Pin,
    sync::Arc,
    task::{ready, Context, Poll},
};

use async_trait::async_trait;
use axum_core::extract::FromRequestParts;
use http::{request::Parts, Request, Response};
use pin_project_lite::pin_project;
use tower::{Layer, Service};

use crate::{extract_current_url, extract_header_bool, extract_header_string, headers};

#[derive(Debug, Clone)]
pub struct RequestHeaders {
    pub boosted: bool,
    pub current_url: Option<http::Uri>,
    pub history_restore: bool,
    pub prompt: Option<String>,
    pub target: Option<String>,
    pub trigger_name: Option<String>,
    pub trigger: Option<String>,
}

impl RequestHeaders {
    fn from_parts(parts: &Parts) -> Self {
        let boosted = extract_header_bool(parts, headers::HX_BOOSTED);
        let current_url = extract_current_url(parts);
        let history_restore = extract_header_bool(parts, headers::HX_HISTORY_RESTORE_REQUEST);
        let prompt = extract_header_string(parts, headers::HX_PROMPT);
        let target = extract_header_string(parts, headers::HX_TARGET);
        let trigger_name = extract_header_string(parts, headers::HX_TRIGGER_NAME);
        let trigger = extract_header_string(parts, headers::HX_TRIGGER);

        Self {
            boosted,
            current_url,
            history_restore,
            prompt,
            target,
            trigger_name,
            trigger,
        }
    }
}

#[derive(Debug, Clone, Default)]
struct InnerResHeaders {}

#[derive(Debug, Clone)]
pub struct ResponseHeaders {
    inner: Arc<InnerResHeaders>,
}

/// Extractor for htmx middleware.
#[derive(Debug)]
pub struct Htmx {
    pub req: RequestHeaders,
    pub res: ResponseHeaders,
}

#[async_trait]
impl<S> FromRequestParts<S> for Htmx {
    type Rejection = std::convert::Infallible;

    async fn from_request_parts(parts: &mut Parts, _: &S) -> Result<Self, Self::Rejection> {
        let req = RequestHeaders::from_parts(parts);
        let inner = parts
            .extensions
            .get::<Arc<InnerResHeaders>>()
            .expect("htmx extension is missing, are you using HtmxLayer middleware?")
            .clone();
        let res = ResponseHeaders { inner };

        Ok(Self { req, res })
    }
}

#[derive(Clone)]
pub struct HtmxService<S> {
    inner: S,
}

impl<S, Req, Res> Service<Request<Req>> for HtmxService<S>
where
    S: Service<Request<Req>, Response = Response<Res>>,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = private::ResponseFuture<S::Future>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, mut req: Request<Req>) -> Self::Future {
        let hs = ResponseHeaders {
            inner: Arc::default(),
        };
        req.extensions_mut().insert(hs.clone());

        private::ResponseFuture {
            fut: self.inner.call(req),
            hs,
        }
    }
}

/// Layer that applies [`Htmx`] middleware.
#[derive(Clone)]
pub struct HtmxLayer;

impl<S> Layer<S> for HtmxLayer {
    type Service = HtmxService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        HtmxService { inner }
    }
}

mod private {
    use super::*;

    pin_project! {
        pub struct ResponseFuture<F> {
            #[pin]
            pub(super) fut: F,
            pub(super) hs: ResponseHeaders,
        }
    }

    impl<F, Res, Err> Future for ResponseFuture<F>
    where
        F: Future<Output = Result<Response<Res>, Err>>,
    {
        type Output = F::Output;

        fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
            let this = self.project();
            let res = ready!(this.fut.poll(cx))?;

            dbg!(this.hs);

            Poll::Ready(Ok(res))
        }
    }
}
