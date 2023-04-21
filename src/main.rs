use axum::{routing::get, Router};
use chrono::{Local, SecondsFormat};

mod config;

#[tokio::main]
async fn main() {
    launch_info();
    let app = Router::new().route("/alive", get(alive));
    axum::Server::bind(&config::CONFIG.addr)
        .serve(app.into_make_service())
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
