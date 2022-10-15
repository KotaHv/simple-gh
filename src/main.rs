use std::io::Write;

use dotenvy::dotenv;
use log::LevelFilter;

#[macro_use]
extern crate rocket;
#[macro_use]
extern crate log;

mod config;
mod fairing;
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
    println!();
    println!("=================== Starting Simple-Gh ===================");
    println!();
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
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S.%3f"),
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
    rocket::build()
        .mount("/", routes![alive])
        .mount("/gh", gh::routes())
        .manage(create_client())
        .manage(config::init_config())
        .attach(fairing::Logging())
        .attach(fairing::BackgroundTask())
}
