#![allow(dead_code)]

use std::{
    future::Future,
    pin::Pin,
    sync::{Arc, Mutex},
    task::{ready, Context, Poll},
};

use async_trait::async_trait;
use axum_core::{extract::FromRequestParts, response::Response};
use http::{request::Parts, HeaderValue, Request};
use pin_project_lite::pin_project;
use tower::{Layer, Service};

use crate::{
    extract_current_url, extract_header_bool, extract_header_string, headers, HxError, HxLocation,
    HxPushUrl, HxRedirect, HxRefresh, HxReplaceUrl, HxReselect, HxResponseTrigger, HxReswap,
    HxRetarget,
};

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
struct InnerResHeaders {
    location: Option<HxLocation>,
    push_url: Option<HxPushUrl>,
    redirect: Option<HxRedirect>,
    refresh: Option<HxRefresh>,
    replace_url: Option<HxReplaceUrl>,
    reswap: Option<HxReswap>,
    retarget: Option<HxRetarget>,
    reselect: Option<HxReselect>,
    trigger: Option<HxResponseTrigger>,
}

/// Control of the response headers.
#[derive(Debug, Clone)]
pub struct ResponseHeaders {
    inner: Arc<Mutex<InnerResHeaders>>,
}

impl ResponseHeaders {
    fn guard(&self, call: impl FnOnce(&mut InnerResHeaders)) {
        if let Ok(mut inner) = self.inner.lock() {
            call(&mut inner);
        }
    }

    /// Sets `HX-Location` header.
    pub fn set_location(&self, location: impl Into<HxLocation>) -> &Self {
        self.guard(|hs| _ = hs.location.replace(location.into()));
        self
    }

    /// Sets `HX-Push-Url` header.
    pub fn set_push_url(&self, push_url: impl Into<HxPushUrl>) -> &Self {
        self.guard(|hs| _ = hs.push_url.replace(push_url.into()));
        self
    }

    /// Sets `HX-Redirect` header.
    pub fn set_redirect(&self, redirect: impl Into<HxRedirect>) -> &Self {
        self.guard(|hs| _ = hs.redirect.replace(redirect.into()));
        self
    }

    /// Sets `HX-Refresh` header.
    pub fn set_refresh(&self, refresh: impl Into<HxRefresh>) -> &Self {
        self.guard(|hs| _ = hs.refresh.replace(refresh.into()));
        self
    }

    /// Sets `HX-Replace-Url` header.
    pub fn set_replace_url(&self, replace_url: impl Into<HxReplaceUrl>) -> &Self {
        self.guard(|hs| _ = hs.replace_url.replace(replace_url.into()));
        self
    }

    /// Sets `HX-Reswap` header.
    pub fn set_reswap(&self, reswap: impl Into<HxReswap>) -> &Self {
        self.guard(|hs| _ = hs.reswap.replace(reswap.into()));
        self
    }

    /// Sets `HX-Retarget` header.
    pub fn set_retarget(&self, retarget: impl Into<HxRetarget>) -> &Self {
        self.guard(|hs| _ = hs.retarget.replace(retarget.into()));
        self
    }

    /// Sets `HX-Reselect` header.
    pub fn set_reselect(&self, reselect: impl Into<HxReselect>) -> &Self {
        self.guard(|hs| _ = hs.reselect.replace(reselect.into()));
        self
    }

    /// Sets `HX-Trigger*` headers
    pub fn set_trigger(&self, trigger: impl Into<HxResponseTrigger>) -> &Self {
        self.guard(|hs| _ = hs.trigger.replace(trigger.into()));
        self
    }
}

/// Extractor for htmx middleware.
#[derive(Debug)]
pub struct Htmx {
    /// Request headers.
    pub req: RequestHeaders,
    /// Handle to set response headers.
    pub res: ResponseHeaders,
}

#[async_trait]
impl<S> FromRequestParts<S> for Htmx {
    type Rejection = std::convert::Infallible;

    async fn from_request_parts(parts: &mut Parts, _: &S) -> Result<Self, Self::Rejection> {
        let req = RequestHeaders::from_parts(parts);
        let res = parts
            .extensions
            .get::<ResponseHeaders>()
            .expect("htmx extension is missing, are you using HtmxLayer middleware?")
            .clone();

        Ok(Self { req, res })
    }
}

#[derive(Clone)]
pub struct HtmxService<S> {
    inner: S,
}

impl<S, Req> Service<Request<Req>> for HtmxService<S>
where
    S: Service<Request<Req>, Response = Response>,
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
#[derive(Clone, Default)]
pub struct HtmxLayer {
    _priv: (),
}

impl HtmxLayer {
    pub fn new() -> Self {
        Self { _priv: () }
    }
}

impl<S> Layer<S> for HtmxLayer {
    type Service = HtmxService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        HtmxService { inner }
    }
}

pub mod private {
    use axum_core::body::Body;
    use http::StatusCode;

    use super::*;

    pin_project! {
        pub struct ResponseFuture<F> {
            #[pin]
            pub(super) fut: F,
            pub(super) hs: ResponseHeaders,
        }
    }

    impl<F> ResponseFuture<F> {}

    impl<F, Err> Future for ResponseFuture<F>
    where
        F: Future<Output = Result<Response, Err>>,
    {
        type Output = Result<Response, Err>;

        fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
            let this = self.project();
            let mut res = ready!(this.fut.poll(cx))?;

            let Ok(mut hs) = this.hs.inner.lock() else {
                return Poll::Ready(Ok(res));
            };

            let out = if let Err(err) = apply(&mut res, &mut hs) {
                Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .body(Body::from(err.to_string()))
                    .unwrap()
            } else {
                res
            };

            Poll::Ready(Ok(out))
        }
    }
}

fn apply<Res>(res: &mut Response<Res>, hs: &mut InnerResHeaders) -> Result<(), HxError> {
    use crate::headers as hxs;

    if let Some(h) = hs.location.take() {
        let val = HeaderValue::from_maybe_shared(h.into_header_with_options()?)?;
        res.headers_mut().append(hxs::HX_LOCATION, val);
    }

    if let Some(h) = hs.push_url.take() {
        let val = HeaderValue::from_maybe_shared(h.0.to_string())?;
        res.headers_mut().append(hxs::HX_PUSH_URL, val);
    }

    if let Some(h) = hs.redirect.take() {
        let val = HeaderValue::from_maybe_shared(h.0.to_string())?;
        res.headers_mut().append(hxs::HX_REDIRECT, val);
    }

    if let Some(h) = hs.refresh.take() {
        if h.0 {
            res.headers_mut()
                .append(hxs::HX_REFRESH, HeaderValue::from_static("true"));
        }
    }

    if let Some(h) = hs.replace_url.take() {
        let val = HeaderValue::from_maybe_shared(h.0.to_string())?;
        res.headers_mut().append(hxs::HX_REPLACE_URL, val);
    }

    if let Some(h) = hs.reswap.take() {
        res.headers_mut().append(hxs::HX_RESWAP, h.0.into());
    }

    if let Some(h) = hs.retarget.take() {
        let val = HeaderValue::from_maybe_shared(h.0)?;
        res.headers_mut().append(hxs::HX_RETARGET, val);
    }

    if let Some(h) = hs.reselect.take() {
        let val = HeaderValue::from_maybe_shared(h.0)?;
        res.headers_mut().append(hxs::HX_RESELECT, val);
    }

    if let Some(h) = hs.trigger.take() {
        let (name, value) = h.into_header_name_value()?;
        res.headers_mut().append(name, value);
    }

    Ok(())
}
