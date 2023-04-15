use chrono::Local;
use chrono::SecondsFormat;
use env_logger::Builder;
use log::Level;
use warp::log::{Info, Log};
use yansi::{Color, Paint};

use crate::config::CONFIG;
use crate::util::get_ip;

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

pub fn warp_log_custom(name: &'static str) -> Log<impl Fn(Info<'_>) + Copy> {
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
            "{ip} {method} {path} => {status} \"{referer}\" [{ua}] {elapsed:?}",
            ip = ip.fg(Color::Red).bold(),
            path = path.fg(Color::Red).underline(),
            status = status.fg(Color::Red).dimmed()
        );
    })
}
