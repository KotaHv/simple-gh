use chrono::{DateTime, Utc};
use mime_guess;
use std::{ffi::OsStr, fs::Metadata, path::PathBuf};
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
    tokio::fs::remove_file(filepath).await.ok();
    tokio::fs::remove_file(typepath(filepath)).await.ok();
}

pub fn create_date(metadata: &Metadata) -> DateTime<Utc> {
    DateTime::from(metadata.created().unwrap_or(metadata.modified().unwrap()))
}
