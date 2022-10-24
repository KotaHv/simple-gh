use std::io::Write;

use config::init_config;
use dotenvy::dotenv;
use log::LevelFilter;
use rocket::{http::Status, State};

#[macro_use]
extern crate rocket;
#[macro_use]
extern crate log;

mod config;
mod fairing;
mod gh;
mod task;
mod util;

#[rocket::main]
async fn main() -> Result<(), rocket::Error> {
    launch_info();
    dotenv().ok();
    init_logger();
    let config = init_config();
    let task_jh = task::background_task(config.clone()).await;
    let _ = rocket::build()
        .mount("/", routes![alive])
        .mount("/gh", gh::routes())
        .manage(create_client())
        .manage(config)
        .manage(task_jh)
        .attach(fairing::Logging())
        .launch()
        .await;
    warn!("simple-gh process exited!");
    Ok(())
}

fn create_client() -> reqwest::Client {
    reqwest::Client::new()
}

#[get("/alive")]
fn alive(task_jh: &State<std::thread::JoinHandle<()>>) -> Result<String, Status> {
    if task_jh.is_finished() {
        error!("background task failed");
        return Err(Status::InternalServerError);
    }
    debug!("background task success");
    Ok(chrono::Local::now().to_string())
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
        .filter(Some("_"), LevelFilter::Error)
        .filter(Some("rocket::shield::shield"), LevelFilter::Warn)
        .filter(Some("rocket::server"), LevelFilter::Warn)
        .filter(Some("reqwest::connect"), LevelFilter::Warn)
        .init();
}
