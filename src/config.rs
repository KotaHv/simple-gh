use std::{
    fmt,
    net::{IpAddr, Ipv4Addr, SocketAddr},
    path::{Path, PathBuf},
};

use figment::{providers::Env, Figment};
use is_terminal::IsTerminal;
use once_cell::sync::Lazy;
use serde::{de::Visitor, Deserialize, Deserializer};

const PREFIX: &'static str = "SIMPLE_GH_";

pub static CONFIG: Lazy<Config> = Lazy::new(|| init_config());

#[derive(Debug)]
pub enum LogStyle {
    Auto,
    Always,
    Never,
}

impl Default for LogStyle {
    fn default() -> Self {
        Self::Auto
    }
}

impl LogStyle {
    pub fn is_color(&self) -> bool {
        match self {
            LogStyle::Auto => std::io::stdout().is_terminal(),
            LogStyle::Always => true,
            LogStyle::Never => false,
        }
    }
}

impl<'de> Deserialize<'de> for LogStyle {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?.to_lowercase();
        match s.as_str() {
            "auto" => Ok(LogStyle::Auto),
            "always" => Ok(LogStyle::Always),
            "never" => Ok(LogStyle::Never),
            _ => Err(serde::de::Error::unknown_field(
                &s,
                &["auto", "always", "never"],
            )),
        }
    }
}

#[derive(Deserialize, Debug)]
#[serde(default)]
pub struct Log {
    pub level: String,
    pub style: LogStyle,
}

impl Default for Log {
    fn default() -> Self {
        Log {
            level: Log::level(),
            style: LogStyle::default(),
        }
    }
}

impl Log {
    fn level() -> String {
        "simple=info".to_string()
    }
}

#[derive(Deserialize, Debug)]
#[serde(default)]
pub struct Cache {
    pub path: PathBuf,
    #[serde(deserialize_with = "deserialize_with_size")]
    pub max: u64,
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
    let config = Figment::from(Env::prefixed(PREFIX))
        .merge(Env::prefixed(PREFIX).split("_"))
        .extract::<Config>();
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
