use std::io::{self, ErrorKind};
use std::path::PathBuf;
use std::thread;

use rocket::tokio::{
    self,
    fs::{create_dir_all, read_dir, DirEntry},
};

use crate::config::Config;
use crate::util;

pub async fn backgroud_task(config: Config) -> thread::JoinHandle<()> {
    info!(target:"BackgroundTask","Starting Backgroud Task");
    let cache_time = chrono::Duration::seconds(config.cache_time as i64);
    let cache_path = config.cache_path.clone();
    let max_cache = config.max_cache;

    let runtime = tokio::runtime::Runtime::new().unwrap();
    thread::Builder::new()
        .name("job-scheduler".to_string())
        .spawn(move || {
            use job_scheduler_ng::{Job, JobScheduler};
            let _runtime_guard = runtime.enter();

            let mut sched = JobScheduler::new();

            sched.add(Job::new("*/10 * * * * *".parse().unwrap(), || {
                runtime.spawn(handle_backgroud_task(
                    cache_time,
                    cache_path.clone(),
                    max_cache,
                ));
            }));

            loop {
                sched.tick();
                runtime.block_on(async move {
                    tokio::time::sleep(tokio::time::Duration::from_millis(10_000)).await
                });
            }
        })
        .expect("Error spawning job scheduler thread")
}

async fn handle_backgroud_task(
    cache_time: chrono::Duration,
    cache_path: PathBuf,
    max_cache: u64,
) -> io::Result<()> {
    let mut entries = match read_dir(&cache_path).await {
        Ok(entries) => entries,
        Err(e) => {
            error!("{:?}:{e}", e.kind());
            if e.kind() == ErrorKind::NotFound {
                error!("mkdir: {:?}", cache_path);
                create_dir_all(&cache_path).await.ok();
            }
            return Err(e);
        }
    };
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
    if cache_size > max_cache {
        warn!("Exceed the maximum cache");
        debug!("{files:?}");
        files.sort_by(|a, b| a.1.cmp(&b.1));
        debug!("{files:?}");
        for (file, _, size) in files.iter() {
            warn!("delete file {:?}", file.file_name());
            util::remove_file(&file.path()).await;
            cache_size -= size;
            if cache_size <= max_cache {
                break;
            }
        }
    }

    Ok(())
}
