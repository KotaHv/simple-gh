use std::fmt;
use std::net::{IpAddr, Ipv4Addr};
use std::path::PathBuf;
use std::{net::SocketAddr, path::Path};

use figment::providers::Serialized;
use figment::{providers::Env, Figment};
use once_cell::sync::Lazy;
use serde::de::Visitor;
use serde::{Deserialize, Deserializer, Serialize};

pub static CONFIG: Lazy<Config> = Lazy::new(|| init_config());

#[derive(Deserialize, Debug, Serialize)]
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
        "simple=info".to_string()
    }

    fn style() -> String {
        "always".to_string()
    }
}

#[derive(Deserialize, Debug, Serialize)]
pub struct Cache {
    #[serde(default = "Cache::path")]
    pub path: PathBuf,
    #[serde(default = "Cache::max", deserialize_with = "deserialize_with_size")]
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

#[derive(Deserialize, Debug, Serialize)]
pub struct Config {
    #[serde(deserialize_with = "deserialize_with_size")]
    pub file_max: u64,
    pub token: Option<String>,
    pub log: Log,
    pub addr: SocketAddr,
    pub cache: Cache,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            file_max: Config::file_max(),
            token: None,
            log: Log::default(),
            addr: Config::addr(),
            cache: Cache::default(),
        }
    }
}

impl Config {
    fn file_max() -> u64 {
        byte_unit::Byte::from_str("24MiB").unwrap().get_bytes()
    }
    fn addr() -> SocketAddr {
        SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 3030)
    }
}

fn deserialize_with_size<'de, D>(deserializer: D) -> Result<u64, D::Error>
where
    D: Deserializer<'de>,
{
    struct SizeVisitor;

    impl<'de> Visitor<'de> for SizeVisitor {
        type Value = u64;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("unsigned integer or string with units of bytes")
        }

        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok(byte_unit::Byte::from_str(v).unwrap().get_bytes())
        }

        fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok(byte_unit::Byte::from_str(&v).unwrap().get_bytes())
        }

        fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok(v)
        }
    }
    deserializer.deserialize_any(SizeVisitor)
}

pub fn init_config() -> Config {
    let config = Figment::from(Serialized::defaults(Config::default()))
        .merge(Env::prefixed("SIMPLE_GH_"))
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
