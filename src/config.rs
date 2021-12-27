use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Deserialize, Serialize, Debug, Default)]
#[serde(default)]
pub struct Config {
    pub modules: ModuleConfig,
    pub server: ServerConfig,
}

#[derive(Deserialize, Serialize, Debug, Default)]
#[serde(default)]
pub struct ModuleConfig {
    #[cfg(windows)]
    pub gsmtc: GsmtcConfig,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(default)]
#[cfg(windows)]
pub struct GsmtcConfig {
    #[serde(default = "bool_true")]
    pub enabled: bool,
    pub filter: GsmtcFilter,
}

#[cfg(windows)]
impl Default for GsmtcConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            filter: Default::default(),
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(tag = "mode", content = "items")]
#[cfg(windows)]
pub enum GsmtcFilter {
    Disabled,
    Include(HashSet<String>),
    Exclude(HashSet<String>),
}

#[cfg(windows)]
impl Default for GsmtcFilter {
    fn default() -> Self {
        let mut set = HashSet::new();
        set.insert("firefox.exe".into());
        set.insert("chrome.exe".into());
        set.insert("msedge.exe".into());
        Self::Exclude(set)
    }
}

#[cfg(windows)]
impl GsmtcFilter {
    pub fn pass_filter(&self, source_model_id: &str) -> bool {
        match self {
            GsmtcFilter::Disabled => true,
            GsmtcFilter::Include(include) => include.contains(source_model_id),
            GsmtcFilter::Exclude(exclude) => !exclude.contains(source_model_id),
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct ServerConfig {
    #[serde(default = "default_port")]
    pub port: u16,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            port: default_port(),
        }
    }
}

#[inline]
fn default_port() -> u16 {
    48457
}

#[inline]
fn bool_true() -> bool {
    true
}

lazy_static::lazy_static! {
    pub static ref CONFIG: Config = {
        let config_path = std::path::PathBuf::from("config.toml");
        match std::fs::read(&config_path).ok().and_then(|file| toml::from_slice(&file).ok()) {
            Some(config) => config,
            None => {
                let conf = Config::default();
                if config_path.exists() {
                    std::fs::rename(&config_path, "config.toml.old").ok();
                }
                std::fs::write(&config_path, toml::to_string(&conf).unwrap()).ok();
                conf
            }
        }
    };
}
