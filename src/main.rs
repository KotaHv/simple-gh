use std::sync::Arc;

use axum::{middleware, routing::get, Router};
use chrono::{Local, SecondsFormat};

#[macro_use]
extern crate log;

mod config;
mod gh;
mod logger;
mod util;

#[tokio::main]
async fn main() {
    launch_info();
    logger::init_logger();
    let client = Arc::new(reqwest::Client::new());
    let app = Router::new()
        .route("/alive", get(alive))
        .nest("/gh", gh::routes(client.clone()))
        .layer(middleware::from_fn(logger::logger_middleware));

    axum::Server::bind(&config::CONFIG.addr)
        .serve(app.into_make_service_with_connect_info::<std::net::SocketAddr>())
        .await
        .unwrap();
}

async fn alive() -> String {
    Local::now().to_rfc3339_opts(SecondsFormat::Millis, false)
}

fn launch_info() {
    println!();
    println!("=================== Starting Simple-Gh ===================");
    println!();
}
