use std::{ffi::OsStr, fs::Metadata, path::PathBuf};

use axum::{
    extract::ConnectInfo,
    http::{header, HeaderMap, Request},
};
use chrono::{DateTime, Utc};
use mime_guess;
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

pub async fn remove_file(filepath: &PathBuf) {
    fs::remove_file(filepath).await.ok();
    fs::remove_file(typepath(filepath)).await.ok();
}

pub fn create_date(metadata: &Metadata) -> DateTime<Utc> {
    DateTime::from(metadata.created().unwrap_or(metadata.modified().unwrap()))
}

pub fn get_ip<B>(req: &Request<B>) -> String {
    let headers = req.headers();
    if let Some(ip) = get_header(headers, "X-Forwarded-For") {
        if let Some(ip) = ip.split_once(",") {
            return ip.0.to_string();
        }
        return ip;
    }
    if let Some(ip) = get_header(headers, "X-Real-IP") {
        return ip;
    }
    if let Some(ConnectInfo(ip)) = req.extensions().get::<ConnectInfo<std::net::SocketAddr>>() {
        return ip.to_string();
    }
    "-".to_string()
}

pub fn get_header(headers: &HeaderMap, key: impl header::AsHeaderName) -> Option<String> {
    if let Some(header) = headers.get(key) {
        if let Ok(header) = header.to_str() {
            return Some(header.to_string());
        }
    }
    None
}

pub fn get_ua(headers: &HeaderMap) -> String {
    match get_header(headers, header::USER_AGENT) {
        Some(ua) => ua,
        None => "-".to_string(),
    }
}
