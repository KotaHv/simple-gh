use std::io::ErrorKind;

use rocket::{
    fairing::{Fairing, Info, Kind},
    tokio, Data, Orbit, Request, Response, Rocket,
};

use crate::config::Config;
use crate::util::get_ip;

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
        info!(target: "routes", "Routes loaded:");
        let mut routes: Vec<_> = rocket.routes().collect();
        routes.sort_by_key(|r| r.uri.path());
        for route in routes {
            if route.rank < 0 {
                info!(target: "routes", "{:<6} {}", route.method, route.uri);
            } else {
                info!(target: "routes", "{:<6} {} [{}]", route.method, route.uri, route.rank);
            }
        }

        let config = rocket.config();
        let addr = format!("http://{}:{}", &config.address, &config.port);
        info!(target: "start", "Rocket has launched from {}", addr);
    }

    async fn on_request(&self, request: &mut Request<'_>, _data: &mut Data<'_>) {
        let method = request.method();
        let uri = request.uri();
        let uri_path = uri.path();
        let uri_path_str = uri_path.url_decode_lossy();
        if LOGGING_ROUTE_BLACKLIST
            .iter()
            .any(|x| uri_path_str.starts_with(x))
        {
            return;
        }
        let ip = get_ip(request);
        match uri.query() {
            Some(q) => {
                info!(target: "request", "{} {} {}?{}", ip, method, uri_path_str, &q[..q.len().min(30)])
            }
            None => info!(target: "request", "{} {} {}", ip, method, uri_path_str),
        };
    }

    async fn on_response<'r>(&self, request: &'r Request<'_>, response: &mut Response<'r>) {
        let uri = request.uri();
        let uri_path = uri.path();
        let uri_path_str = uri_path.url_decode_lossy();
        if LOGGING_ROUTE_BLACKLIST
            .iter()
            .any(|x| uri_path_str.starts_with(x))
        {
            return;
        }
        let ip = get_ip(request);
        let status = response.status();
        if let Some(ref route) = request.route() {
            info!(target: "response", "{} {} => {}", ip, route, status)
        } else {
            info!(target: "response", "{} {}", ip, status)
        }
    }
}

pub struct BackgroundTask();
#[rocket::async_trait]
impl Fairing for BackgroundTask {
    fn info(&self) -> Info {
        Info {
            name: "Background Task",
            kind: Kind::Liftoff,
        }
    }

    async fn on_liftoff(&self, rocket: &Rocket<Orbit>) {
        info!(target:"BackGroundTask","Starting Backgroud Task");
        let config = rocket.state::<Config>().unwrap().clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(10));
            let cache_time = chrono::Duration::seconds(config.cache_time as i64);
            let cache_path = config.cache_path;
            loop {
                match tokio::fs::read_dir(&cache_path).await {
                    Ok(mut entries) => {
                        while let Some(entry) = entries.next_entry().await.unwrap() {
                            let metadata = entry.metadata().await.unwrap();
                            if metadata.is_file() {
                                let create_date =
                                    chrono::DateTime::from(metadata.created().unwrap());
                                let duration = chrono::Utc::now() - create_date;
                                if duration > cache_time {
                                    warn!(target:"BackGroundTask",
                                        "{:?} cache has expired, {:?} > {:?}",
                                        entry.file_name(),
                                        duration,
                                        cache_time
                                    );
                                    tokio::fs::remove_file(entry.path()).await.ok();
                                }
                            }
                        }
                    }
                    Err(e) => {
                        error!("{}:{}", e.kind(), e);
                        if e.kind() == ErrorKind::NotFound {
                            error!("mkdir: {:?}", cache_path);
                            tokio::fs::create_dir_all(&cache_path).await.ok();
                        }
                    }
                }
                interval.tick().await;
            }
        });
    }
}
