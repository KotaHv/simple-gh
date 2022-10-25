use std::{ffi::OsStr, fs::Metadata, path::PathBuf, str, str::FromStr};

use chrono::{DateTime, Utc};
use log::Level;
use rocket::{http::ContentType, tokio, yansi::Paint, Request};

pub fn content_type(ct: &str) -> ContentType {
    ContentType::from_str(ct).unwrap_or(ContentType::Bytes)
}

pub fn content_type_reqwest(res: &reqwest::Response) -> ContentType {
    if let Some(ct) = res.headers().get(reqwest::header::CONTENT_TYPE) {
        if let Ok(ct) = ct.to_str() {
            return content_type(ct);
        }
    }
    ContentType::Bytes
}

pub async fn content_type_typepath(typepath: &PathBuf) -> ContentType {
    match tokio::fs::read(typepath).await {
        Ok(content) => content_type(str::from_utf8(&content).unwrap()),
        Err(_) => {
            if let Some(ext) = typepath.with_extension("").extension() {
                if let Some(ct) = ContentType::from_extension(&ext.to_string_lossy()) {
                    return ct;
                }
            }
            ContentType::Bytes
        }
    }
}

pub fn get_ip(request: &Request) -> String {
    if let Some(ip) = request.headers().get_one("X-Forwarded-For") {
        if let Some(ip) = ip.split_once(",") {
            ip.0.to_string()
        } else {
            ip.to_string()
        }
    } else {
        if let Some(ip) = request.client_ip() {
            ip.to_string()
        } else {
            "Unknown".to_string()
        }
    }
}

pub fn typepath(filepath: &PathBuf) -> PathBuf {
    filepath.with_extension(format!(
        "{}.type",
        filepath
            .extension()
            .unwrap_or(OsStr::new(""))
            .to_string_lossy()
    ))
}

pub async fn remove_file(filepath: &PathBuf) {
    tokio::fs::remove_file(filepath).await.ok();
    tokio::fs::remove_file(typepath(filepath)).await.ok();
}

pub fn create_date(metadata: &Metadata) -> DateTime<Utc> {
    DateTime::from(metadata.created().unwrap_or(metadata.modified().unwrap()))
}

pub fn colored_level(level: Level) -> Paint<Level> {
    match level {
        Level::Trace => Paint::magenta(level),
        Level::Debug => Paint::blue(level),
        Level::Info => Paint::green(level),
        Level::Warn => Paint::yellow(level),
        Level::Error => Paint::red(level),
    }
}

pub fn bold_target<'a>(target: &'a str) -> Paint<&'a str> {
    Paint::new(target).bold()
}
