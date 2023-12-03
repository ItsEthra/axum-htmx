//! Axum extractors for htmx request headers.

use async_trait::async_trait;
use axum_core::extract::FromRequestParts;
use http::request::Parts;

use crate::{
    HX_BOOSTED, HX_CURRENT_URL, HX_HISTORY_RESTORE_REQUEST, HX_PROMPT, HX_REQUEST, HX_TARGET,
    HX_TRIGGER, HX_TRIGGER_NAME,
};

/// The `HX-Boosted` header.
///
/// This is set when a request is made from an element where its parent has the
/// `hx-boost` attribute set to `true`.
///
/// This extractor will always return a value. If the header is not present, it
/// will return `false`.
///
/// See <https://htmx.org/attributes/hx-boost/> for more information.
#[derive(Debug, Clone, Copy)]
pub struct HxBoosted(pub bool);

/// Extracts all bool-like headers
pub(crate) fn extract_header_bool(parts: &Parts, header_name: &str) -> bool {
    matches!(
        parts.headers.get(header_name).map(|x| x.to_str()),
        Some(Ok("true"))
    )
}

/// Extracts all string-like headers
pub(crate) fn extract_header_string(parts: &Parts, header_name: &str) -> Option<String> {
    if let Some(prompt) = parts.headers.get(header_name) {
        return prompt.to_str().ok().map(|s| s.to_owned());
    }

    None
}

#[async_trait]
impl<S> FromRequestParts<S> for HxBoosted
where
    S: Send + Sync,
{
    type Rejection = std::convert::Infallible;

    async fn from_request_parts(parts: &mut Parts, _: &S) -> Result<Self, Self::Rejection> {
        Ok(Self(extract_header_bool(parts, HX_BOOSTED)))
    }
}

pub(crate) fn extract_current_url(parts: &Parts) -> Option<http::Uri> {
    if let Some(url) = parts.headers.get(HX_CURRENT_URL) {
        let url = url
            .to_str()
            .ok()
            .and_then(|url| url.parse::<http::Uri>().ok());
        return url;
    }

    None
}

/// The `HX-Current-Url` header.
///
/// This is set on every request made by htmx itself. As its name implies, it
/// just contains the current url.
///
/// This extractor will always return a value. If the header is not present, or extractor fails to parse the url
/// it will return `None`.
#[derive(Debug, Clone)]
pub struct HxCurrentUrl(pub Option<http::Uri>);

#[async_trait]
impl<S> FromRequestParts<S> for HxCurrentUrl
where
    S: Send + Sync,
{
    type Rejection = std::convert::Infallible;

    async fn from_request_parts(parts: &mut Parts, _: &S) -> Result<Self, Self::Rejection> {
        Ok(Self(extract_current_url(parts)))
    }
}

/// The `HX-History-Restore-Request` header.
///
/// This extractor will always return a value. If the header is not present,
/// it will return `false`.
#[derive(Debug, Clone, Copy)]
pub struct HxHistoryRestoreRequest(pub bool);

#[async_trait]
impl<S> FromRequestParts<S> for HxHistoryRestoreRequest
where
    S: Send + Sync,
{
    type Rejection = std::convert::Infallible;

    async fn from_request_parts(parts: &mut Parts, _: &S) -> Result<Self, Self::Rejection> {
        Ok(Self(extract_header_bool(parts, HX_HISTORY_RESTORE_REQUEST)))
    }
}

/// The `HX-Prompt` header.
///
/// This is set when a request is made from an element that has the `hx-prompt`
/// attribute set. The value will contain the string input by the user.
///
/// This extractor will always return a value. If the header is not present, it
/// will return `None`.
#[derive(Debug, Clone)]
pub struct HxPrompt(pub Option<String>);

#[async_trait]
impl<S> FromRequestParts<S> for HxPrompt
where
    S: Send + Sync,
{
    type Rejection = std::convert::Infallible;

    async fn from_request_parts(parts: &mut Parts, _: &S) -> Result<Self, Self::Rejection> {
        Ok(Self(extract_header_string(parts, HX_PROMPT)))
    }
}

/// The `HX-Request` header.
///
/// This is set on every request made by htmx itself. It won't be present on
/// requests made manually, or by other libraries.
///
/// This extractor will always return a value. If the header is not present, it
/// will return `false`.
#[derive(Debug, Clone, Copy)]
pub struct HxRequest(pub bool);

#[async_trait]
impl<S> FromRequestParts<S> for HxRequest
where
    S: Send + Sync,
{
    type Rejection = std::convert::Infallible;

    async fn from_request_parts(parts: &mut Parts, _: &S) -> Result<Self, Self::Rejection> {
        Ok(Self(extract_header_bool(parts, HX_REQUEST)))
    }
}

/// The `HX-Target` header.
///
/// This is set when a request is made from an element that has the `hx-target`
/// attribute set. The value will contain the target element's id. If the
/// id does not exist on the page, the value will be None.
///
/// This extractor will always return a value. If the header is not present, it
/// will return `None`.
#[derive(Debug, Clone)]
pub struct HxTarget(pub Option<String>);

#[async_trait]
impl<S> FromRequestParts<S> for HxTarget
where
    S: Send + Sync,
{
    type Rejection = std::convert::Infallible;

    async fn from_request_parts(parts: &mut Parts, _: &S) -> Result<Self, Self::Rejection> {
        Ok(Self(extract_header_string(parts, HX_TARGET)))
    }
}

/// The `HX-Trigger-Name` header.
///
/// This is set when a request is made from an element that has the `hx-trigger`
/// attribute set. The value will contain the trigger element's name. If the
/// name does not exist on the page, the value will be None.
///
/// This extractor will always return a value. If the header is not present, it
/// will return `None`.
#[derive(Debug, Clone)]
pub struct HxTriggerName(pub Option<String>);

#[async_trait]
impl<S> FromRequestParts<S> for HxTriggerName
where
    S: Send + Sync,
{
    type Rejection = std::convert::Infallible;

    async fn from_request_parts(parts: &mut Parts, _: &S) -> Result<Self, Self::Rejection> {
        Ok(Self(extract_header_string(parts, HX_TRIGGER_NAME)))
    }
}

/// The `HX-Trigger` header.
///
/// This is set when a request is made from an element that has the `hx-trigger`
/// attribute set. The value will contain the trigger element's id. If the
/// id does not exist on the page, the value will be None.
///
/// This extractor will always return a value. If the header is not present, it
/// will return `None`.
#[derive(Debug, Clone)]
pub struct HxTrigger(pub Option<String>);

#[async_trait]
impl<S> FromRequestParts<S> for HxTrigger
where
    S: Send + Sync,
{
    type Rejection = std::convert::Infallible;

    async fn from_request_parts(parts: &mut Parts, _: &S) -> Result<Self, Self::Rejection> {
        Ok(Self(extract_header_string(parts, HX_TRIGGER)))
    }
}
