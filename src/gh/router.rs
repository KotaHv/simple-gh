use std::sync::Arc;

use axum::{
    extract::State,
    http::{header, StatusCode},
    response::{IntoResponse, Response},
};
use reqwest::Client;
use tokio::fs;

use super::extract::GHPath;
use super::reqwest::Request;
use super::CONFIG;
use crate::util;
use crate::CustomError;

struct GHResponse<T> {
    body: T,
    ctype: String,
}

impl<T> IntoResponse for GHResponse<T>
where
    T: IntoResponse,
{
    fn into_response(self) -> Response {
        let mut res = self.body.into_response();
        res.headers_mut()
            .insert(header::CONTENT_TYPE, self.ctype.parse().unwrap());
        res
    }
}

pub async fn get_gh(
    GHPath(gh_path): GHPath,
    State(client): State<Arc<Client>>,
) -> Result<Response, CustomError> {
    let filepath = gh_path.replace("/", "_");
    let filepath = CONFIG.cache.path.join(filepath);
    let typepath = util::typepath(&filepath);
    match fs::read(&filepath).await {
        Ok(content) => {
            debug!("{filepath:?} is exists");
            let content_type = util::content_type_typepath(&typepath).await;
            return Ok(GHResponse {
                body: content,
                ctype: content_type,
            }
            .into_response());
        }
        Err(e) => {
            if e.kind() != std::io::ErrorKind::NotFound {
                error!("{filepath:?}: {e}")
            }
        }
    }
    let req = Request::new(client, &gh_path);
    let res = req.head().await?;
    match res.content_length() {
        Some(content_length) => {
            if content_length > CONFIG.file_max {
                let reason = format!(
                    "file size: {} > {}",
                    byte_unit::Byte::from_bytes(content_length)
                        .get_appropriate_unit(true)
                        .to_string(),
                    byte_unit::Byte::from_bytes(CONFIG.file_max)
                        .get_appropriate_unit(true)
                        .to_string()
                );
                return Err(CustomError::new(reason, StatusCode::PAYLOAD_TOO_LARGE));
            }
        }
        None => return Err(CustomError::reason(format!("{:#?}", res.headers()))),
    }
    let status_code = res.status();
    let is_success = status_code.is_success();
    let content_type = match res.headers().get(reqwest::header::CONTENT_TYPE) {
        Some(ct) => ct.to_str().unwrap(),
        None => "application/octet-stream",
    };
    let res = req.get().await?;
    let content = res.bytes().await.unwrap();
    if is_success {
        if let Ok(_) = fs::write(&filepath, &content).await {
            fs::write(&typepath, &content_type).await.ok();
        }
    }

    Ok(GHResponse {
        body: content,
        ctype: content_type.to_string(),
    }
    .into_response())
}
