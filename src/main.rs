use std::{sync::Arc, thread};

use chrono::Local;
use dotenvy::dotenv;
use warp::{
    http::StatusCode,
    log::{Info, Log},
    Filter,
};
use yansi::{Color, Paint};

#[macro_use]
extern crate log;

mod config;
mod gh;
mod task;
mod util;

use util::get_ip;

#[tokio::main]
async fn main() {
    launch_info();
    dotenv().ok();
    let config = Arc::new(config::init_config());
    pretty_env_logger::formatted_timed_builder()
        .parse_filters(&config.log.level)
        .parse_write_style(&config.log.style)
        .init();
    let log = log_custom("simple-gh");
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

fn log_custom(name: &'static str) -> Log<impl Fn(Info<'_>) + Copy> {
    warp::log::custom(move |info: Info| {
        let ip = Paint::cyan(get_ip(&info));
        let method = Paint::green(info.method());
        let path = Paint::blue(info.path());
        let status = Paint::yellow(info.status());
        let referer = info.referer().unwrap_or("-");
        let elapsed = info.elapsed();
        if status.inner().is_success() {
            return info!(
                target: name,
                "{ip} {method} {path} => {status} \"{referer}\" {elapsed:?}"
            );
        }
        let ua = info.user_agent().unwrap_or("no ua");
        let ua = Paint::magenta(ua);
        warn!(
            target: name,
            "[{now}] {ip} {method} {path} => {status} \"{referer}\" [{ua}] {elapsed:?}",
            now = Paint::cyan(Local::now()).bold(),
            ip = ip.fg(Color::Red).bold(),
            path = path.fg(Color::Red).underline(),
            status = status.fg(Color::Red).dimmed()
        );
    })
}
