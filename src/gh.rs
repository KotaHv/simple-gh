// use std::io::Read;
use std::path::PathBuf;

use rocket::{
    fs::NamedFile,
    http::{ContentType, Status},
    tokio::fs::write,
    Route, State,
};

use crate::util;

pub fn routes() -> Vec<Route> {
    routes![get_gh]
}

#[derive(Responder)]
enum GhResponse {
    File(Option<NamedFile>),
    Spider(Vec<u8>, ContentType),
    Stream(Vec<u8>),
}

#[get("/<github_path..>")]
async fn get_gh(github_path: PathBuf, client: &State<reqwest::Client>) -> (Status, GhResponse) {
    let filepath = github_path.to_str().unwrap().replace("/", "_");
    if std::path::Path::new(&filepath).exists() {
        return (
            Status::Ok,
            GhResponse::File(NamedFile::open(&filepath).await.ok()),
        );
    }
    let res = client
        .get(format!(
            "https://raw.githubusercontent.com/{}",
            github_path.to_str().unwrap()
        ))
        .send()
        .await
        .unwrap();
    let is_success = res.status().is_success();
    let status_code = Status::new(res.status().as_u16());
    let content_type_option = util::content_type(&res);
    let content: bytes::Bytes = res.bytes().await.unwrap();
    let data: Vec<u8> = content.to_vec();
    if is_success {
        write(&filepath, &data).await.unwrap();
    }
    if let Some(content_type) = content_type_option {
        (
            status_code,
            GhResponse::Spider(data, ContentType::new(content_type.0, content_type.1)),
        )
    } else {
        (status_code, GhResponse::Stream(data))
    }
}
