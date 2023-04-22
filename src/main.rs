use std::sync::Arc;

use axum::{middleware, routing::get, Router};
use chrono::{Local, SecondsFormat};
use tokio::signal::{
    ctrl_c,
    unix::{signal, SignalKind},
};

#[macro_use]
extern crate log;

mod config;
mod gh;
mod logger;
mod task;
mod util;

#[tokio::main]
async fn main() {
    launch_info();
    logger::init_logger();
    info!("listening on http://{}", config::CONFIG.addr);
    let client = Arc::new(reqwest::Client::new());
    let (task_jh, task_cancel) = task::init_background_task();
    let app = Router::new()
        .route("/alive", get(alive))
        .nest("/gh", gh::routes(client.clone()))
        .layer(middleware::from_fn(logger::logger_middleware));

    let server = axum::Server::bind(&config::CONFIG.addr)
        .serve(app.into_make_service_with_connect_info::<std::net::SocketAddr>());
    tokio::select! {
        _ = server => {},
        _ = shutdown_signal() => {}
    }
    task_cancel.cancel();
    task_jh.await.unwrap();
}

async fn shutdown_signal() -> std::io::Result<()> {
    let mut sigterm = signal(SignalKind::terminate())?;
    let mut sigint = signal(SignalKind::interrupt())?;
    let mut sigquit = signal(SignalKind::quit())?;
    tokio::select! {
        _ = sigint.recv() => {
            info!("SIGINT received; starting forced shutdown");
        },
        _ = sigterm.recv() => {
            info!("SIGTERM received; starting graceful shutdown");
        },
        _ = sigquit.recv() => {
            info!("SIGQUIT received; starting forced shutdown");
        },
        _ = ctrl_c() => {
            info!("SIGINT received; starting forced shutdown");
        }
    }
    Ok(())
}

async fn alive() -> String {
    Local::now().to_rfc3339_opts(SecondsFormat::Millis, false)
}

fn launch_info() {
    println!();
    println!("=================== Starting Simple-Gh ===================");
    println!();
}
