use crate::{cfg_unix, cfg_windows};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashSet,
    fs,
    path::{Path, PathBuf},
    sync::OnceLock,
};
use tracing::warn;

#[derive(Deserialize, Serialize, Debug, Default, Clone)]
#[serde(default)]
pub struct Config {
    #[serde(default = "bool_true")]
    #[cfg(windows)]
    pub no_autostart: bool,
    pub modules: ModuleConfig,
    pub server: ServerConfig,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ServerConfig {
    #[serde(default, flatten)]
    pub bind: BindConfig,
    #[serde(default = "default_custom_theme_path")]
    pub custom_theme_path: String,
    #[serde(default = "default_custom_script_path")]
    pub custom_script_path: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(untagged)]
pub enum BindConfig {
    Single { port: u16 },
    Multiple { bind: Vec<std::net::SocketAddr> },
}

impl Default for BindConfig {
    fn default() -> Self {
        Self::Single { port: 48457 }
    }
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            bind: BindConfig::default(),
            custom_theme_path: default_custom_theme_path(),
            custom_script_path: default_custom_script_path(),
        }
    }
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

    #[cfg(unix)]
    #[cfg_attr(unix, serde(default))]
    pub dbus: DbusConfig,
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
                filter: GsmtcFilter::default(),
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

cfg_unix! {
    #[derive(Deserialize, Serialize, Debug, Clone)]
    #[serde(default)]
    pub struct DbusConfig {
        #[serde(default = "bool_true")]
        pub enabled: bool,
        pub destinations: Vec<String>,
    }

    impl Default for DbusConfig {
        fn default() -> Self {
            Self {
                enabled: true,
                destinations: vec!["org.mpris.MediaPlayer2.spotify".to_owned()],
            }
        }
    }
}

static CURRENT_CONFIG_PATH: OnceLock<PathBuf> = OnceLock::new();

lazy_static::lazy_static! {
    pub static ref CONFIG: Config = {
        loop {
            match read_config() {
                Ok((path, config)) => {
                    CURRENT_CONFIG_PATH.get_or_init(|| path);
                    break config
                },
                Err(None) => {
                    warn!("Didn't find any config at any location, creating default one at default location");
                    let conf = Config::default();

                    let path = default_config_paths()[0].clone(); // can't move out of array
                    save_config(&conf, &path).ok();
                    CURRENT_CONFIG_PATH.get_or_init(|| path);

                    break conf
                }
                Err(Some((loc, err))) => {
                    #[cfg(windows)]
                    if !crate::win_setup::should_replace_invalid_config(&loc, &err) {
                        continue; // try again
                    }

                    warn!("Config at {} was invalid - replacing with default config", loc.display());
                    if loc.exists() {
                        fs::rename(&loc, loc.with_file_name("config.toml.old")).ok();
                    }
                    let conf = Config::default();

                    save_config(&conf, &loc).ok();
                    CURRENT_CONFIG_PATH.get_or_init(|| loc);

                    break conf
                }
            }
        }
    };
}

pub fn current_config_path() -> &'static Path {
    CURRENT_CONFIG_PATH.get_or_init(|| default_config_paths()[0].clone())
}

#[cfg(windows)]
fn default_config_paths() -> [PathBuf; 2] {
    [PathBuf::from("config.toml"), {
        let mut appdata = PathBuf::from(
            std::env::var_os("APPDATA").unwrap_or_else(|| "~\\AppData\\Roaming".into()),
        );
        appdata.push("CurrentSong2/config.toml");
        appdata
    }]
}

#[cfg(unix)]
fn default_config_paths() -> [PathBuf; 1] {
    [{
        let mut cfg_home =
            PathBuf::from(std::env::var_os("XDG_CONFIG_HOME").unwrap_or_else(|| {
                let mut cfg_home = std::env::var_os("HOME").unwrap_or_else(|| "~".into());
                cfg_home.push("/.config");
                cfg_home
            }));
        cfg_home.push("CurrentSong2/config.toml");
        cfg_home
    }]
}

#[allow(clippy::result_large_err)]
fn read_config() -> Result<(PathBuf, Config), Option<(PathBuf, toml::de::Error)>> {
    let locations = default_config_paths();
    let mut first_err = None;

    for loc in &locations {
        let Ok(file) = fs::read_to_string(loc) else {
            continue;
        };
        match toml::from_str(&file) {
            Ok(c) => return Ok((loc.clone(), c)),
            Err(e) => {
                warn!(error = %e, "Found config at {} but couldn't read it", loc.display());
                if first_err.is_none() {
                    first_err = Some((loc.clone(), e));
                }
            }
        }
    }
    warn!("Couldn't find a single config");

    Err(first_err)
}

pub fn save_config(config: &Config, path: &Path) -> anyhow::Result<()> {
    // create the parent directory if it doesn't exist first
    if let Some(dir) = path.parent() {
        if !dir.exists() || !dir.is_dir() {
            if let Err(e) = std::fs::create_dir_all(dir) {
                warn!(
                    error = %e,
                    "Failed to create directory containing the config ({})",
                    dir.display()
                );
            }
        }
    }

    Ok(std::fs::write(path, toml::to_string(config)?)?)
}
