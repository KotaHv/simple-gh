use std::path::PathBuf;
use std::{net::SocketAddr, path::Path};

use figment::{providers::Env, Figment};
use once_cell::sync::Lazy;
use serde::Deserialize;

pub static CONFIG: Lazy<Config> = Lazy::new(|| init_config());

#[derive(Deserialize, Debug)]
pub struct Log {
    #[serde(default = "Log::level")]
    pub level: String,
    #[serde(default = "Log::style")]
    pub style: String,
}

impl Default for Log {
    fn default() -> Self {
        Log {
            level: Log::level(),
            style: Log::style(),
        }
    }
}

impl Log {
    fn level() -> String {
        "INFO".to_string()
    }

    fn style() -> String {
        "always".to_string()
    }
}

#[derive(Deserialize, Debug)]
pub struct Cache {
    #[serde(default = "Cache::path")]
    pub path: PathBuf,
    #[serde(default = "Cache::max")]
    pub max: u64,
    #[serde(default = "Cache::expiry")]
    pub expiry: u32,
}

impl Default for Cache {
    fn default() -> Self {
        Cache {
            path: Cache::path(),
            max: Cache::max(),
            expiry: Cache::expiry(),
        }
    }
}
impl Cache {
    fn path() -> PathBuf {
        Path::new("cache").to_owned()
    }
    fn max() -> u64 {
        byte_unit::Byte::from_str("512MiB").unwrap().get_bytes()
    }
    fn expiry() -> u32 {
        60 * 60 * 24
    }
}

#[derive(Deserialize, Debug)]
pub struct Config {
    #[serde(default = "config_default::cache_path")]
    pub cache_path: PathBuf,
    #[serde(default = "config_default::file_max")]
    pub file_max: u64,
    #[serde(default = "config_default::max_cache")]
    pub max_cache: u64,
    #[serde(default = "config_default::cache_time")]
    pub cache_time: u32,
    #[serde(default)]
    pub token: Option<String>,
    #[serde(default)]
    pub log: Log,
    #[serde(default = "config_default::addr")]
    pub addr: SocketAddr,
    #[serde(default)]
    pub cache: Cache,
}

pub fn init_config() -> Config {
    let config = Figment::from(Env::prefixed("SIMPLE_GH_"))
        .merge(Env::prefixed("SIMPLE_GH_").split("_"))
        .extract();
    match config {
        Ok(config) => {
            println!("{:#?}", config);
            config
        }
        Err(err) => {
            panic!("{:?}", err);
        }
    }
}

mod config_default {
    use std::{
        net::{IpAddr, Ipv4Addr, SocketAddr},
        path::{Path, PathBuf},
    };

    pub fn cache_path() -> PathBuf {
        Path::new("cache").to_owned()
    }

    pub fn file_max() -> u64 {
        byte_unit::Byte::from_str("24MiB").unwrap().get_bytes()
    }

    pub fn max_cache() -> u64 {
        byte_unit::Byte::from_str("512MiB").unwrap().get_bytes()
    }

    pub fn cache_time() -> u32 {
        60 * 60 * 24
    }

    pub fn addr() -> SocketAddr {
        SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 3030)
    }
}
