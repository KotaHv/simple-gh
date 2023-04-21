use axum::{
    http::{header, Request},
    middleware::Next,
    response::Response,
};
use chrono::{Local, SecondsFormat};
use env_logger::Builder;
use log::Level;
use tokio::time::Instant;
use yansi::{Color, Paint};

use crate::{config::CONFIG, util};

pub fn init_logger() {
    let mut builder = Builder::new();
    builder
        .format(|buf, record| {
            use std::io::Write;
            let target = record.target();
            let level = match record.level() {
                Level::Error => Paint::red("ERROR"),
                Level::Warn => Paint::yellow("WARN"),
                Level::Info => Paint::green("INFO"),
                Level::Debug => Paint::blue("DEBUG"),
                Level::Trace => Paint::magenta("TRACE"),
            };
            let target = Paint::new(target).bold();
            writeln!(
                buf,
                " {} {} {} > {}",
                Local::now().to_rfc3339_opts(SecondsFormat::Millis, false),
                level,
                target,
                record.args(),
            )
        })
        .parse_filters(&CONFIG.log.level)
        .parse_write_style(&CONFIG.log.style)
        .init();
}

pub async fn logger_middleware<B>(req: Request<B>, next: Next<B>) -> Response {
    let path = req.uri().path();
    if path == "/alive" {
        return next.run(req).await;
    }
    let start = Instant::now();
    let headers = req.headers().clone();
    let ip = Paint::cyan(util::get_ip(&req));
    let method = Paint::green(req.method().clone());
    let path = Paint::blue(path.to_string());
    let referer = util::get_header(&headers, header::REFERER).unwrap_or("-".to_string());
    let res = next.run(req).await;
    let elapsed = Instant::now() - start;
    match res.status().is_success() {
        true => {
            info!(
                "{ip} {method} {path} => {status} \"{referer}\" {elapsed:?}",
                status = Paint::yellow(res.status())
            );
        }
        false => {
            warn!(
                "{ip} {method} {path} => {status} \"{referer}\" [{ua}] {elapsed:?}",
                ip = ip.fg(Color::Red).bold(),
                path = path.fg(Color::Red).underline(),
                status = Paint::red(res.status()),
                ua = Paint::magenta(util::get_ua(&headers))
            )
        }
    }

    res
}
