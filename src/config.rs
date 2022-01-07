use serde::{Deserialize, Serialize};
use std::{collections::HashSet, path::PathBuf};

#[derive(Deserialize, Serialize, Debug, Default, Clone)]
#[serde(default)]
pub struct Config {
    pub no_autostart: bool,
    pub modules: ModuleConfig,
    pub server: ServerConfig,
}

#[derive(Deserialize, Serialize, Debug, Default, Clone)]
#[serde(default)]
pub struct ModuleConfig {
    #[cfg(windows)]
    pub gsmtc: GsmtcConfig,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
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

#[derive(Deserialize, Serialize, Debug, Clone)]
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

#[derive(Deserialize, Serialize, Debug, Clone)]
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
    static ref CONFIG_PATH: PathBuf = PathBuf::from("config.toml");
}
lazy_static::lazy_static! {
    pub static ref CONFIG: Config = {
        match read_config() {
            Ok(config) => config,
            Err(e) => {
                event!(Level::WARN, error = %e, "Couldn't read config, creating a new one.");
                let conf = Config::default();
                if CONFIG_PATH.exists() {
                    std::fs::rename(&*CONFIG_PATH, "config.toml.old").ok();
                }
                save_config(&conf).ok();
                conf
            }
        }
    };
}

fn read_config() -> anyhow::Result<Config> {
    let file = fs::read(&*CONFIG_PATH)?;
    Ok(toml::from_slice(&file)?)
}

pub fn save_config(config: &Config) -> anyhow::Result<()> {
    Ok(std::fs::write(&*CONFIG_PATH, toml::to_string(config)?)?)
}
