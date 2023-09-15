use anyhow::Context;
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Router, Server};
use std::net::SocketAddr;
use tower_http::services::ServeDir;
use tracing::info;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::Layer;
use tracing_subscriber::{filter, fmt};

const PORT: u16 = 3000;

mod templates;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(fmt::layer().with_filter(filter::LevelFilter::INFO))
        .init();

    let app = Router::new()
        .nest_service("/", ServeDir::new("public"))
        .route("/template", get(hello));

    info!("Server listening on port {PORT}");
    let addr = SocketAddr::from(([0, 0, 0, 0], PORT));
    Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .context("Error while starting the server")?;

    Ok(())
}

async fn hello() -> Response {
    templates::hello("world").into_response()
}
