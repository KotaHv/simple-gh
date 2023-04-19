use actix_web::{
    get,
    middleware::{self, TrailingSlash},
    web, App, HttpResponse, HttpServer, Responder,
};
use chrono::{Local, SecondsFormat};
use tokio::task::JoinHandle;

#[macro_use]
extern crate log;

mod config;
mod gh;
mod logger;
mod task;
mod util;

#[get("/alive")]
async fn alive(task: web::Data<JoinHandle<()>>) -> impl Responder {
    if task.is_finished() {
        error!("background task failed");
        return HttpResponse::InternalServerError().body("background task failed");
    }
    debug!("background task success");
    HttpResponse::Ok().body(Local::now().to_rfc3339_opts(SecondsFormat::Millis, false))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    launch_info();
    logger::init_logger();
    let client = web::Data::new(reqwest::Client::new());
    let task_jh = web::Data::new(task::background_task().await);
    HttpServer::new(move || {
        App::new()
            .app_data(client.clone())
            .app_data(task_jh.clone())
            .service(alive)
            .service(gh::routes("/gh"))
            .wrap(logger::log_custom())
            .wrap(middleware::NormalizePath::new(TrailingSlash::MergeOnly))
    })
    .bind(config::CONFIG.addr)?
    .run()
    .await
}

fn launch_info() {
    println!();
    println!("=================== Starting Simple-Gh ===================");
    println!();
}
