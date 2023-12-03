use std::error::Error;

use axum::{error_handling::HandleErrorLayer, response::Html, routing::*, Router};
use axum_htmx::middleware::{Htmx, HtmxLayer};
use http::{StatusCode, Uri};
use tokio::{fs::read_to_string, net::TcpListener};
use tower::ServiceBuilder;

async fn click(htmx: Htmx) -> Html<&'static str> {
    htmx.res
        .set_redirect("https://tokio.rs".parse::<Uri>().unwrap());

    Html("<p>Hello</p>")
}

async fn root() -> Html<String> {
    Html(read_to_string("examples/simple.html").await.unwrap())
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn Error>> {
    let layer = ServiceBuilder::new()
        .layer(HandleErrorLayer::new(|_err| async {
            StatusCode::INTERNAL_SERVER_ERROR
        }))
        .layer(HtmxLayer);

    let app = Router::new()
        .route("/", get(root))
        .route("/click", get(click))
        .layer(layer);
    let listener = TcpListener::bind("127.0.0.1:9000").await?;

    axum::serve(listener, app).await?;
    Ok(())
}
