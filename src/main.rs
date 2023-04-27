use chrono::{Local, SecondsFormat};
use rocket::{http::Status, tokio::task::AbortHandle, State};

#[macro_use]
extern crate rocket;
#[macro_use]
extern crate log;

mod config;
mod error;
mod gh;
mod logger;
mod task;
mod util;

pub use config::CONFIG;
pub use error::CustomError;

#[rocket::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    launch_info();
    logger::init();
    let mut config = rocket::Config::from(rocket::Config::figment());
    config.address = CONFIG.addr.ip();
    config.port = CONFIG.addr.port();
    let client = reqwest::Client::new();
    let (task_jh, task_cancel) = task::init_background_task();
    let task_jh_state = task_jh.abort_handle();
    rocket::custom(config)
        .mount("/", routes![alive])
        .mount("/gh", gh::routes())
        .manage(client)
        .manage(task_jh_state)
        .attach(logger::Logging())
        .launch()
        .await?;
    warn!("simple-gh process exited!");
    task_cancel.cancel();
    task_jh.await?;
    Ok(())
}

#[get("/alive")]
fn alive(task_jh: &State<AbortHandle>) -> Result<String, Status> {
    if task_jh.is_finished() {
        error!("background task failed");
        return Err(Status::InternalServerError);
    }
    debug!("background task success");
    Ok(Local::now()
        .to_rfc3339_opts(SecondsFormat::Millis, false)
        .to_string())
}

fn launch_info() {
    println!();
    println!("=================== Starting Simple-Gh ===================");
    println!();
}
