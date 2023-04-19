use actix_web::{http::header, HttpRequest};
use mime_guess;
use std::{ffi::OsStr, path::PathBuf};
use tokio::fs;

pub fn typepath(filepath: &PathBuf) -> PathBuf {
    filepath.with_extension(format!(
        "{}.type",
        filepath
            .extension()
            .unwrap_or(OsStr::new(""))
            .to_string_lossy()
    ))
}

pub async fn content_type_typepath(typepath: &PathBuf) -> String {
    fs::read(typepath)
        .await
        .map(|f| String::from_utf8_lossy(&f).to_string())
        .unwrap_or(
            mime_guess::from_path(typepath.with_extension(""))
                .first_or_octet_stream()
                .to_string(),
        )
}

pub fn get_ip(req: &HttpRequest) -> String {
    let connection_info = req.connection_info();
    if let Some(ip) = connection_info.realip_remote_addr() {
        return ip.to_string();
    }
    if let Some(ip) = connection_info.peer_addr() {
        return ip.to_string();
    }
    "-".to_string()
}

pub fn get_header(req: &HttpRequest, key: impl header::AsHeaderName) -> String {
    if let Some(header) = req.headers().get(key) {
        if let Ok(header) = header.to_str() {
            return header.to_string();
        }
    }
    "-".to_string()
}

pub fn get_ua(req: &HttpRequest) -> String {
    get_header(req, header::USER_AGENT)
}
