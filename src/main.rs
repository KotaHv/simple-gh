use actix_web::{get, web, App, HttpResponse, HttpServer, Responder};
use chrono::{Local, SecondsFormat};

#[macro_use]
extern crate log;

mod config;
mod gh;
mod logger;
mod util;

#[get("/alive")]
async fn alive() -> impl Responder {
    HttpResponse::Ok().body(Local::now().to_rfc3339_opts(SecondsFormat::Millis, false))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    launch_info();
    logger::init_logger();
    HttpServer::new(|| {
        App::new()
            .app_data(web::Data::new(reqwest::Client::new()))
            .service(alive)
            .service(web::scope("/gh").configure(gh::routes))
            .wrap(logger::log_custom())
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
