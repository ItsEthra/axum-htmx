#![cfg_attr(feature = "unstable", feature(doc_cfg))]
#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]

mod error;
pub use error::*;

pub mod extract;

/// Axum middleware. Currently only includes guards to protect partial content.
#[cfg(feature = "middleware")]
#[cfg_attr(feature = "unstable", doc(cfg(feature = "middleware")))]
pub mod middleware {
    mod guard;
    #[doc(inline)]
    pub use guard::*;
}

pub mod headers;
pub mod response;

#[doc(inline)]
pub use headers::*;
