// use std::io::Read;
use std::io::Write;
// use std::path::PathBuf;

use log::LevelFilter;
// use rocket::fs::NamedFile;
// use rocket::http::{ContentType, Status};
// use rocket::tokio::fs::write;
use rocket::State;

#[macro_use]
extern crate rocket;
#[macro_use]
extern crate log;

mod gh;
mod util;

fn create_client() -> reqwest::Client {
    reqwest::Client::new()
}

#[get("/")]
async fn index(client: &State<reqwest::Client>) -> String {
    let res = client.get("https://api.my-ip.io/ip").send().await.unwrap();
    let ip = res.text().await.unwrap();
    info!("{}", ip);
    ip
}

#[get("/alive")]
fn alive() -> String {
    chrono::prelude::Local::now().to_string()
}

// #[derive(Responder)]
// enum GhResponse {
//     File(Option<NamedFile>),
//     Spider(Vec<u8>, ContentType),
// }

// #[get("/<github_path..>")]
// async fn get_gh(github_path: PathBuf, client: &State<reqwest::Client>) -> (Status, GhResponse) {
//     let filepath = github_path.to_str().unwrap().replace("/", "_");
//     if std::path::Path::new(&filepath).exists() {
//         return (
//             Status::Ok,
//             GhResponse::File(NamedFile::open(&filepath).await.ok()),
//         );
//     }
//     let res = client
//         .get(format!(
//             "https://raw.githubusercontent.com/{}",
//             github_path.to_str().unwrap()
//         ))
//         .send()
//         .await
//         .unwrap();
//     let is_success = res.status().is_success();
//     let status_code = Status::new(res.status().as_u16());
//     let mut content_type: Vec<String> = res
//         .headers()
//         .get(reqwest::header::CONTENT_TYPE)
//         .unwrap()
//         .to_str()
//         .unwrap()
//         .to_string()
//         .split("/")
//         .map(String::from)
//         .collect();
//     let content = res.bytes().await.unwrap();
//     let data: Result<Vec<u8>, _> = content.bytes().collect();
//     if is_success {
//         write(&filepath, data.as_ref().unwrap()).await.unwrap();
//     }

//     (
//         status_code,
//         GhResponse::Spider(
//             data.unwrap(),
//             ContentType::new(content_type.swap_remove(0), content_type.swap_remove(0)),
//         ),
//     )
// }

fn launch_info() {
    println!("Starting Simple-Gh");
}

fn init_logger() {
    let env = env_logger::Env::default()
        .filter_or("SIMPLE_LOG", LevelFilter::Info.as_str())
        .write_style_or("SIMPLE_LOG_STYLE", "auto");
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
        .filter(Some("reqwest::connect"), LevelFilter::Warn)
        .init();
}

#[launch]
fn rocket() -> _ {
    launch_info();
    init_logger();

    rocket::build()
        .mount("/", routes![index, alive])
        .mount("/gh", gh::routes())
        .manage(create_client())
}
