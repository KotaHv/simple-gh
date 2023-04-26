use chrono::Local;
use chrono::SecondsFormat;
use env_logger::Builder;
use log::Level;
use rocket::yansi::Paint;

use crate::config::CONFIG;

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
