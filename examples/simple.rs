use std::error::Error;

use axum::{response::Html, routing::*, Router};
use axum_htmx::{
    middleware::{Htmx, HtmxLayer},
    SwapOption,
};
use tokio::{fs::read_to_string, net::TcpListener};

async fn random(htmx: Htmx) -> Html<String> {
    let num = fastrand::u32(0..100);
    let name = htmx.req.prompt.unwrap_or_default();
    htmx.res.set_reswap(SwapOption::AfterBegin);

    Html(format!("<p>{name}: {num}</p>"))
}

async fn root() -> Html<String> {
    Html(read_to_string("examples/simple.html").await.unwrap())
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn Error>> {
    let app = Router::new()
        .route("/", get(root))
        .route("/random", get(random))
        .layer(HtmxLayer::new());
    let listener = TcpListener::bind("127.0.0.1:9000").await?;

    axum::serve(listener, app).await?;
    Ok(())
}
