use std::sync::Arc;

use axum::{
    extract::State,
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use chrono::{Local, SecondsFormat};
use tokio::signal::{
    ctrl_c,
    unix::{signal, SignalKind},
};
use tokio::task::AbortHandle;

#[macro_use]
extern crate log;

mod config;
mod error;
mod gh;
mod logger;
mod task;
mod util;

use crate::error::CustomError;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    launch_info();
    logger::init_logger();
    info!("listening on http://{}", config::CONFIG.addr);
    let client = Arc::new(reqwest::Client::new());
    let (task_jh, task_cancel) = task::init_background_task();
    let task_jh_state = Arc::new(task_jh.abort_handle());
    let app = Router::new()
        .route("/alive", get(alive))
        .with_state(task_jh_state)
        .nest("/gh", gh::routes())
        .with_state(client)
        .layer(logger::LoggerLayer);

    let server = axum::Server::bind(&config::CONFIG.addr)
        .serve(app.into_make_service_with_connect_info::<std::net::SocketAddr>());
    tokio::select! {
        _ = server => {},
        _ = shutdown_signal() => {}
    }
    task_cancel.cancel();
    task_jh.await?;
    Ok(())
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

async fn alive<'a>(State(task): State<Arc<AbortHandle>>) -> Result<Response, CustomError> {
    if task.is_finished() {
        error!("background task failed");
        return Err(CustomError::reason("background task failed"));
    }
    debug!("background task success");
    Ok(Local::now()
        .to_rfc3339_opts(SecondsFormat::Millis, false)
        .into_response())
}

fn launch_info() {
    println!();
    println!("=================== Starting Simple-Gh ===================");
    println!();
}
