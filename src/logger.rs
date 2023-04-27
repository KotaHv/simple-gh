use std::io::{self, Write};

use chrono::Local;
use chrono::SecondsFormat;
use env_logger::fmt::Formatter;
use env_logger::Builder;
use log::Level;
use log::Record;
use rocket::{
    fairing::{Fairing, Info, Kind},
    http::hyper::header,
    tokio::time::Instant,
    yansi::Paint,
    Data, Request, Response,
};

use crate::{
    util::{get_header, get_ip, get_ua},
    CONFIG,
};

pub fn init() {
    let mut builder = Builder::new();
    builder
        .format(|buf, record| {
            let target = record.target();
            if target.starts_with("rocket") {
                return rocket_log(buf, record);
            }
            let target = Paint::default(target).bold();
            let level = colored_level(record.level());
            writeln!(
                buf,
                "{} {} {} > {}",
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

fn rocket_log(buf: &mut Formatter, record: &Record) -> io::Result<()> {
    let target = record.target();
    let indented: bool = target.ends_with('_');
    if indented {
        write!(buf, "   {} ", Paint::default(">>").bold())?
    }
    let level = record
        .target()
        .contains("rocket::launch")
        .then(|| Level::Info)
        .unwrap_or_else(|| record.level());
    match level {
        log::Level::Error if !indented => {
            writeln!(
                buf,
                "{} {}",
                Paint::red("Error:").bold(),
                Paint::red(record.args()).wrap()
            )
        }
        log::Level::Warn if !indented => {
            writeln!(
                buf,
                "{} {}",
                Paint::yellow("Warning:").bold(),
                Paint::yellow(record.args()).wrap()
            )
        }
        log::Level::Info => writeln!(buf, "{}", Paint::blue(record.args()).wrap()),
        log::Level::Trace => writeln!(buf, "{}", Paint::magenta(record.args()).wrap()),
        log::Level::Warn => writeln!(buf, "{}", Paint::yellow(record.args()).wrap()),
        log::Level::Error => writeln!(buf, "{}", Paint::red(record.args()).wrap()),
        log::Level::Debug => {
            write!(buf, "\n{} ", Paint::blue("-->").bold())?;
            if let Some(file) = record.file() {
                write!(buf, "{}", Paint::blue(file))?;
            }

            if let Some(line) = record.line() {
                writeln!(buf, ":{}", Paint::blue(line))?;
            }

            writeln!(buf, "\t{}", record.args())
        }
    }
}

fn colored_level(level: Level) -> Paint<&'static str> {
    match level {
        Level::Error => Paint::red("ERROR"),
        Level::Warn => Paint::yellow("WARN"),
        Level::Info => Paint::green("INFO"),
        Level::Debug => Paint::blue("DEBUG"),
        Level::Trace => Paint::magenta("TRACE"),
    }
}

const LOGGING_ROUTE_DEBUG: [&str; 1] = ["/alive"];
pub struct Logging();
#[rocket::async_trait]
impl Fairing for Logging {
    fn info(&self) -> Info {
        Info {
            name: "Logging",
            kind: Kind::Request | Kind::Response,
        }
    }

    async fn on_request(&self, request: &mut Request<'_>, _data: &mut Data<'_>) {
        request.local_cache(|| Instant::now());
    }

    async fn on_response<'r>(&self, request: &'r Request<'_>, response: &mut Response<'r>) {
        let ip = get_ip(request);
        let method = request.method();
        let path = request.uri().path().as_str();
        let path_query = match request.uri().query() {
            Some(query) => format!("{}?{}", path, query),
            None => path.to_string(),
        };
        let headers = request.headers();
        let referer = get_header(&headers, header::REFERER.as_str()).unwrap_or("-".to_string());
        let status = response.status();
        if status.class().is_success() {
            if LOGGING_ROUTE_DEBUG.iter().any(|x| x == &path) {
                debug!(
                    "{ip} {method} {path} => {status} \"{referer}\" {elapsed:?}",
                    ip = Paint::cyan(ip),
                    method = Paint::green(method),
                    path = Paint::blue(path_query),
                    status = Paint::yellow(status),
                    elapsed = request.local_cache(|| Instant::now()).elapsed()
                )
            } else {
                info!(
                    "{ip} {method} {path} => {status} \"{referer}\" {elapsed:?}",
                    ip = Paint::cyan(ip),
                    method = Paint::green(method),
                    path = Paint::blue(path_query),
                    status = Paint::yellow(status),
                    elapsed = request.local_cache(|| Instant::now()).elapsed()
                )
            }
        } else {
            let ua = get_ua(&headers);
            warn!(
                "{ip} {method} {path} => {status} \"{referer}\" [{ua}] {elapsed:?}",
                ip = Paint::red(ip).bold(),
                method = Paint::green(method),
                path = Paint::red(path_query),
                status = Paint::red(status),
                elapsed = request.local_cache(|| Instant::now()).elapsed(),
                ua = Paint::magenta(ua),
            )
        }
    }
}
