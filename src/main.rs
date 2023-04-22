use std::sync::Arc;

use axum::{middleware, routing::get, Router};
use chrono::{Local, SecondsFormat};
use tokio::{
    signal::{
        ctrl_c,
        unix::{signal, SignalKind},
    },
    sync::oneshot,
};

#[macro_use]
extern crate log;

mod config;
mod gh;
mod logger;
mod task;
mod util;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    launch_info();
    logger::init_logger();
    let client = Arc::new(reqwest::Client::new());
    let (task_jh, task_cancel) = task::init_background_task();
    let (tx, rx) = oneshot::channel();
    let app = Router::new()
        .route("/alive", get(alive))
        .nest("/gh", gh::routes(client.clone()))
        .layer(middleware::from_fn(logger::logger_middleware));

    let graceful = axum::Server::bind(&config::CONFIG.addr)
        .serve(app.into_make_service_with_connect_info::<std::net::SocketAddr>())
        .with_graceful_shutdown(async {
            info!("listening on http://{}", config::CONFIG.addr);
            rx.await.ok();
        });
    tokio::spawn(graceful);
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
    let _ = tx.send(());
    task_cancel.cancel();
    task_jh.await.unwrap();
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
