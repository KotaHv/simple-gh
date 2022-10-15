use std::io::{ErrorKind, Write};

use dotenvy::dotenv;
use log::LevelFilter;
use rocket::tokio;

#[macro_use]
extern crate rocket;
#[macro_use]
extern crate log;

mod config;
mod gh;
mod util;

fn create_client() -> reqwest::Client {
    reqwest::Client::new()
}

#[get("/alive")]
fn alive() -> String {
    chrono::prelude::Local::now().to_string()
}

fn launch_info() {
    println!("Starting Simple-Gh");
}

fn init_logger() {
    let env = env_logger::Env::default()
        .filter_or("SIMPLE_GH_LOG_LEVEL", LevelFilter::Info.as_str())
        .write_style_or("SIMPLE_GH_LOG_STYLE", "auto");
    env_logger::Builder::from_env(env)
        .format(|buf, record| {
            writeln!(
                buf,
                "[{}][{}][{}]: {}",
                chrono::Local::now().format("%Y-%m-%dT%H:%M:%S"),
                record.target(),
                record.level(),
                record.args()
            )
        })
        .filter(Some("rocket::launch"), LevelFilter::Warn)
        .filter(Some("_"), LevelFilter::Warn)
        .filter(Some("rocket::shield::shield"), LevelFilter::Warn)
        .filter(Some("rocket::server"), LevelFilter::Warn)
        .filter(Some("reqwest::connect"), LevelFilter::Warn)
        .init();
}

#[launch]
fn rocket() -> _ {
    launch_info();
    dotenv().ok();
    init_logger();
    let config_state = config::init_config();
    let config_clone = config_state.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(10));
        let cache_time = chrono::Duration::seconds(config_clone.cache_time as i64);
        let cache_path = config_clone.cache_path;
        loop {
            // let mut entries_rust = tokio::fs::read_dir(&cache_path).await;
            match tokio::fs::read_dir(&cache_path).await {
                Ok(mut entries) => {
                    while let Some(entry) = entries.next_entry().await.unwrap() {
                        let metadata = entry.metadata().await.unwrap();
                        if metadata.is_file() {
                            let create_date = chrono::DateTime::from(metadata.created().unwrap());
                            let duration = chrono::Utc::now() - create_date;
                            if duration > cache_time {
                                warn!(
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
    rocket::build()
        .mount("/", routes![alive])
        .mount("/gh", gh::routes())
        .manage(create_client())
        .manage(config_state)
}
