use crate::cfg_windows;
use serde::{Deserialize, Serialize};
use std::{collections::HashSet, fs, path::PathBuf};
use tracing::{event, Level};

#[derive(Deserialize, Serialize, Debug, Default, Clone)]
#[serde(default)]
pub struct Config {
    pub no_autostart: bool,
    pub modules: ModuleConfig,
    pub server: ServerConfig,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ServerConfig {
    #[serde(default = "default_port")]
    pub port: u16,
    #[serde(default = "default_custom_theme_path")]
    pub custom_theme_path: String,
    #[serde(default = "default_custom_script_path")]
    pub custom_script_path: String,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            port: default_port(),
            custom_theme_path: default_custom_theme_path(),
            custom_script_path: default_custom_script_path(),
        }
    }
}

#[inline]
fn default_port() -> u16 {
    48457
}

#[inline]
fn default_custom_theme_path() -> String {
    "theme.css".to_string()
}

#[inline]
fn default_custom_script_path() -> String {
    "user.js".to_string()
}

#[inline]
fn bool_true() -> bool {
    true
}

#[inline]
fn bool_false() -> bool {
    false
}

#[derive(Deserialize, Serialize, Debug, Default, Clone)]
#[serde(default)]
pub struct ModuleConfig {
    #[serde(default)]
    pub file: FileOutputConfig,
    #[cfg(windows)]
    #[cfg_attr(windows, serde(default))]
    pub gsmtc: GsmtcConfig,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct FileOutputConfig {
    #[serde(default = "bool_false")]
    pub enabled: bool,
    #[serde(default = "default_file_path")]
    pub path: PathBuf,
    #[serde(default = "default_format")]
    pub format: String,
}

fn default_file_path() -> PathBuf {
    "current_song.txt".into()
}

fn default_format() -> String {
    "{artist} - {title}".into()
}

impl Default for FileOutputConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            path: default_file_path(),
            format: default_format(),
        }
    }
}

cfg_windows! {
    #[derive(Deserialize, Serialize, Debug, Clone)]
    #[serde(default)]
    pub struct GsmtcConfig {
        #[serde(default = "bool_true")]
        pub enabled: bool,
        pub filter: GsmtcFilter,
    }

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
    pub enum GsmtcFilter {
        Disabled,
        Include(HashSet<String>),
        Exclude(HashSet<String>),
    }

    impl Default for GsmtcFilter {
        fn default() -> Self {
            let mut set = HashSet::new();
            set.insert("firefox.exe".into());
            set.insert("chrome.exe".into());
            set.insert("msedge.exe".into());
            Self::Exclude(set)
        }
    }

    impl GsmtcFilter {
        pub fn pass_filter(&self, source_model_id: &str) -> bool {
            match self {
                GsmtcFilter::Disabled => true,
                GsmtcFilter::Include(include) => include.contains(source_model_id),
                GsmtcFilter::Exclude(exclude) => !exclude.contains(source_model_id),
            }
        }
    }
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
