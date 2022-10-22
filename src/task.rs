use std::io::ErrorKind;
use std::sync::Arc;

use tokio::fs::{create_dir_all, read_dir, DirEntry};
use tokio::time::{interval, Duration};

use crate::config::Config;
use crate::util;

pub fn backgroud_task(config: Arc<Config>) {
    info!(target:"BackGroundTask","Starting Backgroud Task");
    tokio::spawn(handle_backgroud_task(config));
}

async fn handle_backgroud_task(config: Arc<Config>) -> Result<(), std::io::Error> {
    let mut interval = interval(Duration::from_secs(10));
    let cache_time = chrono::Duration::seconds(config.cache_time as i64);
    let cache_path = &config.cache_path;
    loop {
        match read_dir(&cache_path).await {
            Ok(mut entries) => {
                let mut cache_size = 0;
                let mut files: Vec<(DirEntry, chrono::DateTime<chrono::Utc>, u64)> = Vec::new();
                while let Some(entry) = entries.next_entry().await? {
                    let metadata = entry.metadata().await?;
                    if metadata.is_file() {
                        let filepath = entry.path();
                        if filepath.extension().unwrap() == "type" {
                            continue;
                        }
                        let create_date = util::create_date(&metadata);
                        let duration = chrono::Utc::now() - create_date;
                        if duration > cache_time {
                            warn!(target:"BackGroundTask",
                                "{:?} cache has expired, {duration:?} > {cache_time:?}",entry.file_name()
                            );
                            util::remove_file(&filepath).await;
                            continue;
                        }
                        let file_size = metadata.len();
                        cache_size += file_size;
                        files.push((entry, create_date, file_size));
                    }
                }
                if cache_size > config.max_cache {
                    warn!("Exceed the maximum cache");
                    debug!("{files:?}");
                    files.sort_by(|a, b| a.1.cmp(&b.1));
                    debug!("{files:?}");
                    for (file, _, size) in files.iter() {
                        warn!("delete file {:?}", file.file_name());
                        util::remove_file(&file.path()).await;
                        cache_size -= size;
                        if cache_size <= config.max_cache {
                            break;
                        }
                    }
                }
            }
            Err(e) => {
                error!("{:?}:{e}", e.kind());
                if e.kind() == ErrorKind::NotFound {
                    error!("mkdir: {:?}", cache_path);
                    create_dir_all(&cache_path).await.ok();
                }
            }
        }
        interval.tick().await;
    }
}
