use std::io::ErrorKind;

use tokio::fs::{create_dir_all, read_dir, DirEntry};
use tokio::task;
use tokio::time::sleep;

use crate::config::CONFIG;
use crate::util;

pub async fn background_task() -> task::JoinHandle<()> {
    info!(target:"BackgroundTask","Starting Background Task");
    task::spawn(async {
        let cache_time = chrono::Duration::seconds(CONFIG.cache_time as i64);
        loop {
            let mut entries = match read_dir(&CONFIG.cache_path).await {
                Ok(entries) => entries,
                Err(e) => {
                    error!("{:?}:{e}", e.kind());
                    if e.kind() == ErrorKind::NotFound {
                        error!("mkdir: {:?}", CONFIG.cache_path);
                        create_dir_all(&CONFIG.cache_path).await.ok();
                    }
                    continue;
                }
            };
            let mut cache_size = 0;
            let mut files: Vec<(DirEntry, chrono::DateTime<chrono::Utc>, u64)> = Vec::new();
            while let Ok(Some(entry)) = entries.next_entry().await {
                if let Ok(metadata) = entry.metadata().await {
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
            }
            if cache_size > CONFIG.max_cache {
                warn!("Exceed the maximum cache");
                debug!("{files:?}");
                files.sort_by(|a, b| a.1.cmp(&b.1));
                debug!("{files:?}");
                for (file, _, size) in files.iter() {
                    warn!("delete file {:?}", file.file_name());
                    util::remove_file(&file.path()).await;
                    cache_size -= size;
                    if cache_size <= CONFIG.max_cache {
                        break;
                    }
                }
            }
            sleep(tokio::time::Duration::from_secs(10)).await
        }
    })
}
