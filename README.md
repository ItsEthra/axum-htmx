# axum-htmx

<!-- markdownlint-disable -->
<div align="right">
<a href="https://crates.io/crates/axum-htmx">
    <img src="https://img.shields.io/crates/v/axum-htmx?style=flat-square" alt="crates.io badge">
</a>
<a href="https://docs.rs/axum-htmx/latest/">
    <img src="https://img.shields.io/docsrs/axum-htmx?style=flat-square" alt="docs.rs badge">
</a>
</div>
<br>
<!-- markdownlint-enable -->

`axum-htmx` is a small extension library providing extractors, responders, and
 request guards for [htmx](https://htmx.org/) headers within
 [axum](https://github.com/tokio-rs/axum).

## Table of Contents

- [axum-htmx](#axum-htmx)
  - [Table of Contents](#table-of-contents)
  - [Getting Started](#getting-started)
  - [Extractors](#extractors)
  - [Responders](#responders)
  - [Request Guards](#request-guards)
  - [Examples](#examples)
    - [Example: Extractors](#example-extractors)
    - [Example: Responders](#example-responders)
    - [Example: Router Guard](#example-router-guard)
  - [Feature Flags](#feature-flags)
  - [Contributing](#contributing)
  - [License](#license)

## Getting Started

Run `cargo add axum-htmx` to add the library to your project.

## Extractors

All of the [htmx request headers](https://htmx.org/reference/#request_headers)
have a supported extractor. Extractors are infallible, meaning they will always
succeed and never return an error. In the case where a header is not present,
the extractor will return `None` or `false` dependant on the expected return
type.

| Header                       | Extractor                   | Value                       |
|------------------------------|-----------------------------|-----------------------------|
| `HX-Boosted`                 | [`HxBoosted`]               | [`bool`]                    |
| `HX-Current-URL`             | [`HxCurrentUrl`]            | [`Option<Uri>`]             |
| `HX-History-Restore-Request` | [`HxHistoryRestoreRequest`] | [`bool`]                    |
| `HX-Prompt`                  | [`HxPrompt`]                | [`Option<String>`]          |
| `HX-Request`                 | [`HxRequest`]               | [`bool`]                    |
| `HX-Target`                  | [`HxTarget`]                | [`Option<String>`]          |
| `HX-Trigger-Name`            | [`HxTriggerName`]           | [`Option<String>`]          |
| `HX-Trigger`                 | [`HxTrigger`]               | [`Option<String>`]          |

[`HxBoosted`]: extract/struct.HxBoosted.html
[`HxCurrentUrl`]: extract/struct.HxCurrentUrl.html
[`HxHistoryRestoreRequest`]: extract/struct.HxHistoryRestoreRequest.html
[`HxPrompt`]: extract/struct.HxPrompt.html
[`HxRequest`]: extract/struct.HxRequest.html
[`HxTarget`]: extract/struct.HxTarget.html
[`HxTriggerName`]: extract/struct.HxTriggerName.html
[`HxTrigger`]: extract/struct.HxTrigger.html

## Responders

All of the [htmx response headers](https://htmx.org/reference/#response_headers)
have a supported responder. A responder is a basic type that implements
`IntoResponseParts`, allowing you to simply and safely apply the HX-* headers to
any of your responses.

| Header                    | Responder             | Value            |
|---------------------------|-----------------------|------------------|
| `HX-Location`             | [`HxLocation`]        | [`Uri`]          |
| `HX-Push-Url`             | [`HxPushUrl`]         | [`Uri`]          |
| `HX-Redirect`             | [`HxRedirect`]        | [`Uri`]          |
| `HX-Refresh`              | [`HxRefresh`]         | [`bool`]         |
| `HX-Replace-Url`          | [`HxReplaceUrl`]      | [`Uri`]          |
| `HX-Reswap`               | [`HxReswap`]          | [`SwapOption`]   |
| `HX-Retarget`             | [`HxRetarget`]        | [`String`]       |
| `HX-Reselect`             | [`HxReselect`]        | [`String`]       |
| `HX-Trigger`              | [`HxResponseTrigger`] | [`HxEvent`]      |
| `HX-Trigger-After-Settle` | [`HxResponseTrigger`] | [`HxEvent`]      |
| `HX-Trigger-After-Swap`   | [`HxResponseTrigger`] | [`HxEvent`]      |

[`HxLocation`]: response/struct.HxLocation.html
[`HxPushUrl`]: response/struct.HxPushUrl.html
[`HxRedirect`]: response/struct.HxRedirect.html
[`HxRefresh`]: response/struct.HxRefresh.html
[`HxReplaceUrl`]: response/struct.HxReplaceUrl.html
[`HxReswap`]: response/struct.HxReswap.html
[`HxRetarget`]: response/struct.HxRetarget.html
[`HxReselect`]: response/struct.HxReselect.html
[`HxResponseTrigger`]: response/struct.HxResponseTrigger.html
[`HxResponseTrigger`]: response/struct.HxResponseTrigger.html
[`HxResponseTrigger`]: response/struct.HxResponseTrigger.html

[`Uri`]: https://docs.rs/http/latest/http/uri/struct.Uri.html
[`SwapOption`]: response/enum.SwapOption.html
[`HxEvent`]: response/struct.HxEvent.html

## Request Guards

__Requires features `guards`.__

In addition to the extractors, there is also a route-wide layer request guard
for the `HX-Request` header. This will redirect any requests without the header
to "/" by default.

_It should be noted that this is NOT a replacement for an auth guard. A user can
trivially set the `HX-Request` header themselves. This is merely a convenience
for preventing users from receiving partial responses without context. If you
need to secure an endpoint you should be using a proper auth system._

## Examples

### Example: Extractors

In this example, we'll look for the `HX-Boosted` header, which is set when
applying the [hx-boost](https://htmx.org/attributes/hx-boost/) attribute to an
element. In our case, we'll use it to determine what kind of response we send.

When is this useful? When using a templating engine, like
[minijinja](https://github.com/mitsuhiko/minijinja), it is common to extend
different templates from a `_base.html` template. However, htmx works by sending
partial responses, so extending our `_base.html` would result in lots of extra
data being sent over the wire.

If we wanted to swap between pages, we would need to support both full template
responses and partial responses _(as the page can be accessed directly or
through a boosted anchor)_, so we look for the `HX-Boosted` header and extend
from a `_partial.html` template instead.

```rust
use axum::response::IntoResponse;
use axum_htmx::HxBoosted;

async fn get_index(HxBoosted(boosted): HxBoosted) -> impl IntoResponse {
    if boosted {
        // Send a template extending from _partial.html
    } else {
        // Send a template extending from _base.html
    }
}
```

### Example: Responders

We can trigger any event being listened to by the DOM using an [htmx
trigger](https://htmx.org/attributes/hx-trigger/) header.

```rust
use axum_htmx::HxResponseTrigger;

// When we load our page, we will trigger any event listeners for "my-event.
async fn index() -> (HxResponseTrigger, &'static str) {
    // Note: As HxResponseTrigger only implements `IntoResponseParts`, we must
    // return our trigger first here.
    (
        HxResponseTrigger::normal(["my-event", "second-event"]),
        "Hello, world!",
    )
}
```

`htmx` also allows arbitrary data to be sent along with the event, which we can
use via the `serde` feature flag and the `HxEvent` type.

```rust
use serde_json::json;

// Note that we are using `HxResponseTrigger` from the `axum_htmx::serde` module
// instead of the root module.
use axum_htmx::{HxEvent, HxResponseTrigger};

async fn index() -> (HxResponseTrigger, &'static str) {
    let event = HxEvent::new_with_data(
        "my-event",
        // May be any object that implements `serde::Serialize`
        json!({"level": "info", "message": {
            "title": "Hello, world!",
            "body": "This is a test message.",
        }}),
    )
    .unwrap();

    // Note: As HxResponseTrigger only implements `IntoResponseParts`, we must
    // return our trigger first here.
    (HxResponseTrigger::normal([event]), "Hello, world!")
}
```

### Example: Router Guard

```rust
use axum::Router;
use axum_htmx::HxRequestGuardLayer;

fn router_one() -> Router {
    Router::new()
        // Redirects to "/" if the HX-Request header is not present
        .layer(HxRequestGuardLayer::default())
}

fn router_two() -> Router {
    Router::new()
        .layer(HxRequestGuardLayer::new("/redirect-to-this-route"))
}
```

## Feature Flags

<!-- markdownlint-disable -->
| Flag     | Default  | Description                                                | Dependencies                                |
|----------|----------|------------------------------------------------------------|---------------------------------------------|
| `guards` | Disabled | Adds request guard layers.                                 | `tower`, `futures-core`, `pin-project-lite` |
| `serde`  | Disabled | Adds serde support for the `HxEvent` and `LocationOptions` | `serde`, `serde_json`                       |
<!-- markdownlint-enable -->

## Contributing

Contributions are always welcome! If you have an idea for a feature or find a
bug, let me know. PR's are appreciated, but if it's not a small change, please
open an issue first so we're all on the same page!

## License

`axum-htmx` is dual-licensed under either

- **[MIT License](/LICENSE-MIT)**
- **[Apache License, Version 2.0](/LICENSE-APACHE)**

at your option.
