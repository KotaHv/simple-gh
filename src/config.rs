use std::net::SocketAddr;
use std::path::PathBuf;

use figment::{providers::Env, Figment};
use once_cell::sync::Lazy;
use serde::Deserialize;

pub static CONFIG: Lazy<Config> = Lazy::new(|| init_config());

#[derive(Deserialize, Debug)]
pub struct Log {
    #[serde(default = "config_default::log_level")]
    pub level: String,
    #[serde(default = "config_default::log_style")]
    pub style: String,
}

impl Default for Log {
    fn default() -> Self {
        Log {
            level: config_default::log_level(),
            style: config_default::log_style(),
        }
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

    pub fn log_level() -> String {
        "INFO".to_string()
    }

    pub fn log_style() -> String {
        "always".to_string()
    }
}
