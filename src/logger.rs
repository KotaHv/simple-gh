use chrono::Local;
use chrono::SecondsFormat;
use env_logger::Builder;
use log::Level;
use rocket::{
    fairing::{Fairing, Info, Kind},
    http::hyper::header,
    tokio::time::Instant,
    yansi::Paint,
    Data, Orbit, Request, Response, Rocket,
};

use crate::{
    util::{get_header, get_ip, get_ua},
    CONFIG,
};

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

const LOGGING_ROUTE_DEBUG: [&str; 1] = ["/alive"];
pub struct Logging();
#[rocket::async_trait]
impl Fairing for Logging {
    fn info(&self) -> Info {
        Info {
            name: "Logging",
            kind: Kind::Liftoff | Kind::Request | Kind::Response,
        }
    }

    async fn on_liftoff(&self, rocket: &Rocket<Orbit>) {
        info!("Routes loaded:");
        let mut routes: Vec<_> = rocket.routes().collect();
        routes.sort_by_key(|r| r.uri.path());
        for route in routes {
            if route.rank < 0 {
                info!(
                    "{:<6} {}",
                    Paint::green(&route.method),
                    Paint::blue(&route.uri)
                );
            } else {
                info!(
                    "{:<6} {} [{}]",
                    Paint::green(&route.method),
                    Paint::blue(&route.uri),
                    Paint::cyan(&route.rank)
                );
            }
        }

        let config = rocket.config();
        let addr = format!("http://{}:{}", &config.address, &config.port);
        info!("Rocket has launched from {}", Paint::blue(addr));
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
