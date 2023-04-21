use std::net::SocketAddr;

use axum::{routing::get, Router};
use chrono::{Local, SecondsFormat};

#[tokio::main]
async fn main() {
    let app = Router::new().route("/alive", get(alive));
    let addr = SocketAddr::from(([127, 0, 0, 1], 3030));
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn alive() -> String {
    Local::now().to_rfc3339_opts(SecondsFormat::Millis, false)
}
