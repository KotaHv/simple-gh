use std::sync::Arc;

use chrono::Local;
use dotenvy::dotenv;
use warp::{
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

fn alive() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("alive")
        .and(warp::get())
        .map(|| Local::now().to_string())
}

fn launch_info() {
    println!();
    println!("=================== Starting Simple-Gh ===================");
    println!();
}

fn log_custom(name: &'static str) -> Log<impl Fn(Info<'_>) + Copy> {
    warp::log::custom(move |info: Info| {
        let ip;
        match info.remote_addr() {
            Some(addr) => ip = addr.to_string(),
            None => ip = "Unknown".to_string(),
        }
        let ip = Paint::cyan(ip);
        let method = Paint::green(info.method());
        let path = Paint::blue(info.path());
        let status = Paint::yellow(info.status());
        let referer = info.referer().unwrap_or("-");
        let elapsed = info.elapsed();
        if status.inner().is_success() {
            info!(
                target: name,
                "{ip} {method} {path} => {status} \"{referer}\" {elapsed:?}"
            );
        } else {
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
        }
    })
}

#[tokio::main]
async fn main() {
    launch_info();
    dotenv().ok();
    let config = Arc::new(config::init_config());
    pretty_env_logger::formatted_builder()
        .parse_filters(&config.log.level)
        .parse_write_style(&config.log.style)
        .init();
    let log = log_custom("simple-gh");
    task::backgroud_task(config.clone()).await;
    let client = reqwest::Client::new();
    let gh = gh::routes(client.clone(), config.clone()).with(log);
    let routes = alive().or(warp::path("gh").and(gh));
    warp::serve(routes).run(config.addr).await;
}
