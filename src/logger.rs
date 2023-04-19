use actix_web::dev::ServiceResponse;
use actix_web::middleware::Logger;
use chrono::Local;
use chrono::SecondsFormat;
use env_logger::Builder;
use log::Level;
use yansi::{Color, Paint};

use crate::config::CONFIG;
use crate::util::{get_ip, get_ua};

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

pub fn log_custom() -> Logger {
    fn format() -> String {
        let referer = "%{Referer}i";
        let elapsed = "%T";
        format!("%{{STATUS}}xo \"{referer}\" {elapsed}")
    }

    fn error(res: &ServiceResponse) -> String {
        let ip = Paint::cyan(get_ip(res.request()));
        let method = Paint::green(res.request().method());
        let path = Paint::blue(res.request().path());
        let status = Paint::yellow(res.response().status());
        if res.status().is_success() {
            format!("{ip} {method} {path} => {status}")
        } else {
            let ua = get_ua(res.request());
            format!(
                "{ip} {method} {path} => {status} [{ua:?}]",
                ip = ip.fg(Color::Red).bold(),
                path = path.fg(Color::Red).underline(),
                status = status.fg(Color::Red).dimmed()
            )
        }
    }

    Logger::new(format().as_str())
        .custom_response_replace("STATUS", |res| error(res))
        .exclude("/alive")
}
