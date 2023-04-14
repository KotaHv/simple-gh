use std::{sync::Arc, thread};

use chrono::Local;
use dotenvy::dotenv;
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
    let config = Arc::new(config::init_config());
    logger::init_logger(config.clone());
    let log = logger::warp_log_custom("simple-gh");
    let task_jh = task::background_task(config.clone()).await;
    let client = reqwest::Client::new();
    let gh_routes = gh::routes(client.clone(), config.clone()).with(log);
    let routes = alive_routes(Arc::new(task_jh)).or(warp::path("gh").and(gh_routes));
    warp::serve(routes).run(config.addr).await;
}

fn alive_routes(
    task_jh: Arc<thread::JoinHandle<()>>,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("alive")
        .and(warp::get())
        .map(move || task_jh.clone())
        .map(alive)
}

fn alive(task_jh: Arc<thread::JoinHandle<()>>) -> Box<dyn warp::Reply> {
    if task_jh.is_finished() {
        error!("background task failed");
        return Box::new(StatusCode::INTERNAL_SERVER_ERROR);
    }
    debug!("background task success");
    Box::new(Local::now().to_string())
}

fn launch_info() {
    println!();
    println!("=================== Starting Simple-Gh ===================");
    println!();
}
