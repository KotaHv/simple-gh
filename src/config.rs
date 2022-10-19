use std::fs::create_dir_all;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug)]
pub struct Log {
    pub level: String,
    pub style: String,
}

#[derive(Clone, Debug)]
pub struct Config {
    pub cache_path: PathBuf,
    pub file_max: u64,
    pub max_cache: u64,
    pub cache_time: u32,
    pub token: String,
    pub log: Log,
    pub addr: SocketAddr,
}

pub fn init_config() -> Config {
    let log = Config::log();
    let cache_path = Config::cache_path();
    let (file_max, file_max_str) = Config::file_max();
    let (max_cache, max_cache_str) = Config::max_cache();
    if file_max > max_cache {
        panic!(
            "SIMPLE_GH_FILE_MAX({}) cannot be greater than SIMPLE_GH_MAX_CACHE({})",
            file_max_str, max_cache_str
        );
    }
    let cache_time = Config::cache_time();
    let token = Config::token();
    let addr = Config::addr();
    Config {
        cache_path,
        file_max,
        max_cache,
        cache_time,
        token,
        log,
        addr,
    }
}

impl Config {
    fn addr() -> SocketAddr {
        let address = dotenvy::var("SIMPLE_GH_ADDRESS").unwrap_or("127.0.0.1".to_string());
        let port = dotenvy::var("SIMPLE_GH_PORT").unwrap_or("3030".to_string());
        format!("{address}:{port}").parse::<SocketAddr>().unwrap()
    }
    fn log() -> Log {
        let level = dotenvy::var("SIMPLE_GH_LOG_LEVEL").unwrap_or("info".to_string());
        let style = dotenvy::var("SIMPLE_GH_LOG_STYLE").unwrap_or("auto".to_string());
        Log { level, style }
    }
    fn cache_path() -> PathBuf {
        let cache_str = dotenvy::var("SIMPLE_GH_CACHE_DIR").unwrap_or("cache".to_string());
        let cache_path = Path::new(&cache_str);
        if !cache_path.exists() {
            info!("mkdir: {}", cache_str);
            create_dir_all(cache_path).ok();
        }
        cache_path.to_owned()
    }
    fn file_max() -> (u64, String) {
        let file_max_str = dotenvy::var("SIMPLE_GH_FILE_MAX").unwrap_or("24MiB".to_string());
        let file_max = byte_unit::Byte::from_str(&file_max_str)
            .unwrap()
            .get_bytes();
        (file_max, file_max_str)
    }
    fn max_cache() -> (u64, String) {
        let max_cache_str = dotenvy::var("SIMPLE_GH_MAX_CACHE").unwrap_or("512MiB".to_string());
        let max_cache = byte_unit::Byte::from_str(&max_cache_str)
            .unwrap()
            .get_bytes();
        (max_cache, max_cache_str)
    }
    fn cache_time() -> u32 {
        let cache_time = dotenvy::var("SIMPLE_GH_CACHE_TIME").unwrap_or((60 * 60 * 24).to_string());
        cache_time.parse().unwrap()
    }
    fn token() -> String {
        dotenvy::var("SIMPLE_GH_TOKEN").unwrap_or("".to_string())
    }
}
