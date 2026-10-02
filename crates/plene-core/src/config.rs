//! Settings that last between runs, kept in the config directory.
//!
//! Author: David M. Anderson
//! Built with AI assistance (Claude, Anthropic)

use std::fmt;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::category::Category;

/// What the reader has chosen to keep: the categories left as written, not expanded.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Config {
    pub keep: Vec<Category>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigError(String);

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid config: {}", self.0)
    }
}

impl std::error::Error for ConfigError {}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct ConfigFile {
    #[serde(default)]
    keep: Vec<String>,
}

impl Config {
    /// `config.toml` in plene's directory under the config directory, which is
    /// `$XDG_CONFIG_HOME`, or `~/.config` when that is unset or not absolute.
    pub fn path() -> Option<PathBuf> {
        config_dir().map(|dir| dir.join("config.toml"))
    }

    /// The config in effect: the file at `path()`, or the default when there is none.
    /// Each warning and error names the file.
    pub fn load() -> Result<(Config, Vec<String>), String> {
        match Config::path().filter(|path| path.exists()) {
            Some(path) => Config::load_from(&path),
            None => Ok((Config::default(), Vec::new())),
        }
    }

    pub fn load_from(path: &Path) -> Result<(Config, Vec<String>), String> {
        let text = std::fs::read_to_string(path)
            .map_err(|error| format!("{}: {error}", path.display()))?;
        let (config, warnings) =
            Config::parse(&text).map_err(|error| format!("{}: {error}", path.display()))?;
        let warnings = warnings
            .into_iter()
            .map(|warning| format!("{}: {warning}", path.display()))
            .collect();
        Ok((config, warnings))
    }

    /// Parses a config file. A category that is not one is skipped with a warning, and
    /// one named twice counts once.
    pub fn parse(text: &str) -> Result<(Config, Vec<String>), ConfigError> {
        let file: ConfigFile =
            toml::from_str(text).map_err(|error| ConfigError(error.to_string()))?;
        let mut keep = Vec::new();
        let mut warnings = Vec::new();
        for id in file.keep {
            match id.parse::<Category>() {
                Ok(category) if !keep.contains(&category) => keep.push(category),
                Ok(_) => {}
                Err(unknown) => warnings.push(format!("skipping {unknown}")),
            }
        }
        Ok((Config { keep }, warnings))
    }

    /// The config as a file `parse` reads back, with the categories in their usual order.
    pub fn to_toml(&self) -> String {
        let keep = Category::ALL
            .iter()
            .filter(|category| self.keep.contains(category))
            .map(|category| category.as_str().to_string())
            .collect();
        toml::to_string(&ConfigFile { keep }).expect("a config serializes")
    }

    /// Writes the config to `path`, making its directory if need be. The file is written
    /// beside the real one and moved into place, so a failure cannot leave half a file.
    pub fn save(&self, path: &Path) -> Result<(), String> {
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir).map_err(|error| format!("{}: {error}", dir.display()))?;
        }
        let staging = path.with_extension("toml.tmp");
        std::fs::write(&staging, self.to_toml())
            .and_then(|()| std::fs::rename(&staging, path))
            .map_err(|error| format!("{}: {error}", path.display()))
    }
}

/// `$XDG_CONFIG_HOME/plene`, or `~/.config/plene` when `XDG_CONFIG_HOME` is unset or not
/// absolute, which the XDG spec says to ignore.
pub(crate) fn config_dir() -> Option<PathBuf> {
    let config_home = std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .filter(|path| path.is_absolute())
        .or_else(|| std::env::home_dir().map(|home| home.join(".config")))?;
    Some(config_home.join("plene"))
}
