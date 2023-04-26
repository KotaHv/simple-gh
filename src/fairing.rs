use std::time::SystemTime;

use rocket::{
    fairing::{Fairing, Info, Kind},
    yansi::{Color, Paint},
    Data, Orbit, Request, Response, Rocket,
};

use crate::util::get_ip;

#[derive(Copy, Clone)]
struct TimerStart(Option<SystemTime>);

const LOGGING_ROUTE_BLACKLIST: [&str; 1] = ["/alive"];
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
        request.local_cache(|| TimerStart(Some(SystemTime::now())));
    }

    async fn on_response<'r>(&self, request: &'r Request<'_>, response: &mut Response<'r>) {
        let method = Paint::green(request.method());
        let uri = request.uri();
        let uri_path = uri.path();
        let uri_path_str = uri_path.url_decode_lossy();
        if LOGGING_ROUTE_BLACKLIST
            .iter()
            .any(|x| uri_path_str.starts_with(x))
        {
            return;
        }
        let ip = Paint::cyan(get_ip(request));
        let mut query = "".to_string();
        if let Some(q) = uri.query() {
            query = format!("?{}", q.as_str());
        }
        let uri_path_query = Paint::blue(uri_path_str.to_string() + &query);
        let start_time = request.local_cache(|| TimerStart(None));
        let duration_str = if let Some(Ok(duration)) = start_time.0.map(|st| st.elapsed()) {
            let d_str = format!("{duration:?}");
            response.set_raw_header("X-Response-Time", d_str.clone());
            d_str
        } else {
            "".to_string()
        };
        let status = Paint::yellow(response.status());
        if status.inner().code >= 400 {
            let ua = Paint::yellow(request.headers().get_one("user-agent").unwrap_or("Unknown"));
            error!(
                "{ip} {method} {uri_path_query} => {status} [{ua}] {duration_str}",
                ip = ip.fg(Color::Red).bold(),
                uri_path_query = uri_path_query.fg(Color::Red).underline(),
                status = status.fg(Color::Red).dimmed()
            );
        } else {
            info!("{ip} {method} {uri_path_query} => {status} {duration_str}");
        }
    }
}
