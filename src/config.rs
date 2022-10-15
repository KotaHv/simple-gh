use std::fs::create_dir_all;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug)]
pub struct Config {
    pub cache_path: PathBuf,
    pub file_max: u128,
    pub cache_time: u32,
    pub token: String,
}

pub fn init_config() -> Config {
    let cache_path = Config::cache_path();
    let file_max = Config::file_max();
    let cache_time = Config::cache_time();
    let token = Config::token();
    Config {
        cache_path,
        file_max,
        cache_time,
        token,
    }
}

impl Config {
    fn cache_path() -> PathBuf {
        let cache_str = dotenvy::var("SIMPLE_GH_CACHE_DIR").unwrap_or("cache".to_string());
        let cache_path = Path::new(&cache_str);
        if !cache_path.exists() {
            info!("mkdir: {}", cache_str);
            create_dir_all(cache_path).ok();
        }
        cache_path.to_owned()
    }
    fn file_max() -> u128 {
        let file_max_str = dotenvy::var("SIMPLE_GH_FILE_MAX").unwrap_or("24MiB".to_string());
        let file_max = byte_unit::Byte::from_str(file_max_str).unwrap().get_bytes();
        file_max
    }
    fn cache_time() -> u32 {
        let cache_time = dotenvy::var("SIMPLE_GH_CACHE_TIME").unwrap_or((60 * 60 * 24).to_string());
        cache_time.parse().unwrap()
    }
    fn token() -> String {
        dotenvy::var("SIMPLE_GH_TOKEN").unwrap_or("".to_string())
    }
}
