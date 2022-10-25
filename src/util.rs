use chrono::{DateTime, Utc};
use mime_guess;
use std::{ffi::OsStr, fs::Metadata, path::PathBuf};
use tokio::fs;
use warp::log::Info;

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
    tokio::fs::remove_file(filepath).await.ok();
    tokio::fs::remove_file(typepath(filepath)).await.ok();
}

pub fn create_date(metadata: &Metadata) -> DateTime<Utc> {
    DateTime::from(metadata.created().unwrap_or(metadata.modified().unwrap()))
}

pub fn get_ip(info: &Info) -> String {
    let headers = info.request_headers();
    if let Some(ip) = headers.get("X-Forwarded-For") {
        if let Ok(ip) = ip.to_str() {
            if let Some(ip) = ip.split_once(",") {
                return ip.0.to_string();
            }
            return ip.to_string();
        }
    }
    if let Some(ip) = headers.get("X-Real-IP") {
        if let Ok(ip) = ip.to_str() {
            return ip.to_string();
        }
    }
    match info.remote_addr() {
        Some(ip) => ip.to_string(),
        None => "Unknown".to_string(),
    }
}
