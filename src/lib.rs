#![cfg_attr(feature = "unstable", feature(doc_cfg))]
#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]

pub mod extractors;
#[cfg(feature = "middleware")]
#[cfg_attr(feature = "unstable", doc(cfg(feature = "middleware")))]
pub mod guard;
pub mod headers;
#[cfg(feature = "middleware")]
#[cfg_attr(feature = "unstable", doc(cfg(feature = "middleware")))]
pub mod middleware;
pub mod responders;

#[doc(inline)]
pub use extractors::*;
#[cfg(feature = "middleware")]
#[cfg_attr(feature = "unstable", doc(cfg(feature = "middleware")))]
#[doc(inline)]
pub use guard::*;
#[doc(inline)]
pub use headers::*;
#[doc(inline)]
#[cfg(feature = "middleware")]
#[cfg_attr(feature = "unstable", doc(cfg(feature = "middleware")))]
pub use middleware::*;
#[doc(inline)]
pub use responders::*;
