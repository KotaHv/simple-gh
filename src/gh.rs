use std::path::PathBuf;

use byte_unit::Byte;
use rocket::{
    fs::NamedFile,
    http::{ContentType, Status},
    tokio::fs::write,
    Route, State,
};

use crate::config::Config;
use crate::util;

pub fn routes() -> Vec<Route> {
    routes![get_gh]
}

#[derive(Responder)]
enum GhResponse {
    File(Option<NamedFile>),
    Spider(Vec<u8>, ContentType),
}

#[get("/<github_path..>")]
async fn get_gh(
    github_path: PathBuf,
    client: &State<reqwest::Client>,
    config: &State<Config>,
) -> (Status, GhResponse) {
    let file_str = github_path.to_str().unwrap().replace("/", "_");
    let filepath = config.cache_path.join(&file_str);
    if filepath.exists() {
        info!("{} is exists", &file_str);
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
    let content_type_result = util::content_type(&res);
    let content_length_option = res.content_length();
    let content: bytes::Bytes = res.bytes().await.unwrap();
    let data: Vec<u8> = content.to_vec();
    if is_success {
        if let Some(content_length) = content_length_option {
            if content_length as u128 <= config.file_max {
                write(&filepath, &data).await.unwrap();
            } else {
                warn!(
                    "{} content-length:{} > {}",
                    &file_str,
                    Byte::from_bytes(content_length as u128)
                        .get_appropriate_unit(true)
                        .to_string(),
                    Byte::from_bytes(config.file_max)
                        .get_appropriate_unit(true)
                        .to_string()
                );
            }
        } else {
            warn!("{} content-length is None", &file_str);
        }
    }
    match content_type_result {
        Ok(content_type) => (
            status_code,
            GhResponse::Spider(data, ContentType::new(content_type.0, content_type.1)),
        ),
        Err(content_type_string) => {
            warn!(
                "path={:?}, content-type={}",
                github_path, content_type_string
            );
            (
                status_code,
                GhResponse::Spider(data, ContentType::new(content_type_string, "")),
            )
        }
    }
}
