use std::sync::Arc;

use chrono::{Local, SecondsFormat};
use dotenvy::dotenv;
use tokio::{
    signal,
    sync::oneshot,
    task::{spawn, AbortHandle},
};
use warp::{http::StatusCode, Filter};

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
    dotenv().ok();
    logger::init_logger();
    let log = logger::warp_log_custom("simple-gh");
    let (task_jh, task_cancel) = task::init_background_task();
    let client = reqwest::Client::new();
    let gh_routes = gh::routes(client.clone()).with(log);
    let routes = alive_routes(Arc::new(task_jh.abort_handle())).or(warp::path("gh").and(gh_routes));
    let (tx, rx) = oneshot::channel();
    let (_, server) = warp::serve(routes).bind_with_graceful_shutdown(config::CONFIG.addr, async {
        info!("listening on http://{}", config::CONFIG.addr);
        rx.await.ok();
        info!("SIGINT received; starting forced shutdown");
    });
    spawn(server);
    signal::ctrl_c().await.unwrap();
    let _ = tx.send(());
    task_cancel.cancel();
    task_jh.await.unwrap();
}

type Task = Arc<AbortHandle>;

fn alive_routes(
    task_jh: Task,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("alive")
        .and(warp::get())
        .map(move || task_jh.clone())
        .map(alive)
}

fn alive(task_jh: Task) -> Box<dyn warp::Reply> {
    if task_jh.is_finished() {
        error!("background task failed");
        return Box::new(StatusCode::INTERNAL_SERVER_ERROR);
    }
    debug!("background task success");
    Box::new(Local::now().to_rfc3339_opts(SecondsFormat::Millis, false))
}

fn launch_info() {
    println!();
    println!("=================== Starting Simple-Gh ===================");
    println!();
}
