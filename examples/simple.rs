use std::error::Error;

use axum::{response::Html, routing::*, Router};
use axum_htmx::Htmx;
use tokio::{fs::read_to_string, net::TcpListener};

async fn click(htmx: Htmx) -> Html<&'static str> {
    dbg!(htmx);

    Html("<p>Hello</p>")
}

async fn root() -> Html<String> {
    Html(read_to_string("examples/simple.html").await.unwrap())
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn Error>> {
    let app = Router::new()
        .route("/", get(root))
        .route("/click", get(click))
        .layer(axum_htmx::HtmxLayer);
    let listener = TcpListener::bind("127.0.0.1:9000").await?;

    axum::serve(listener, app).await?;
    Ok(())
}
