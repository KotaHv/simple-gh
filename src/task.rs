use std::io::ErrorKind;

use tokio::{
    fs::{create_dir_all, read_dir, DirEntry},
    task,
    time::{self, sleep},
};
use tokio_util::sync::CancellationToken;

use crate::config::CONFIG;
use crate::util;

pub fn init_background_task() -> (task::JoinHandle<()>, CancellationToken) {
    let cancel = CancellationToken::new();

    (task::spawn(background_task(cancel.clone())), cancel)
}

async fn background_task(stop_signal: CancellationToken) {
    info!("Starting Background Task");
    let cache_time = chrono::Duration::seconds(CONFIG.cache.expiry as i64);
    loop {
        let mut entries = match read_dir(&CONFIG.cache.path).await {
            Ok(entries) => entries,
            Err(e) => {
                error!("{:?}:{e}", e.kind());
                if e.kind() == ErrorKind::NotFound {
                    error!("mkdir: {:?}", CONFIG.cache.path);
                    create_dir_all(&CONFIG.cache.path).await.ok();
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
                    if let Some(extension) = filepath.extension() {
                        if extension == "type" {
                            continue;
                        }
                    }
                    let create_date = util::create_date(&metadata);
                    let duration = chrono::Utc::now() - create_date;
                    if duration > cache_time {
                        warn!(
                            "{:?} cache has expired, {duration:?} > {cache_time:?}",
                            entry.file_name()
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
        if cache_size > CONFIG.cache.max {
            warn!("Exceed the maximum cache");
            debug!("{files:?}");
            files.sort_by(|a, b| a.1.cmp(&b.1));
            debug!("{files:?}");
            for (file, _, size) in files.iter() {
                warn!("delete file {:?}", file.file_name());
                util::remove_file(&file.path()).await;
                cache_size -= size;
                if cache_size <= CONFIG.cache.max {
                    break;
                }
            }
        }
        tokio::select! {
            _ = sleep(time::Duration::from_secs(10)) => {
                continue;
            }

            _ = stop_signal.cancelled() => {
                info!("gracefully shutting down background task");
                break;
            }
        };
    }
}
