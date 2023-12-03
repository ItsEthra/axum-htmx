#![cfg_attr(feature = "unstable", feature(doc_cfg))]
#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]

/// Guard and htmx middleware.
#[cfg(feature = "middleware")]
#[cfg_attr(feature = "unstable", doc(cfg(feature = "middleware")))]
pub mod middleware {
    mod guard;
    pub use guard::*;
    mod htmx;
    pub use htmx::*;
}

pub mod extractors;
pub mod headers;
pub mod responders;

#[doc(inline)]
pub use extractors::*;
#[doc(inline)]
pub use headers::*;
#[cfg(feature = "middleware")]
#[cfg_attr(feature = "unstable", doc(cfg(feature = "middleware")))]
#[doc(inline)]
pub use middleware::*;
#[doc(inline)]
pub use responders::*;
