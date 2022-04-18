//! Theia bot configuration

use crate::error::TheiaError;

use ::serde::Deserialize;
use ::serde_json::Value;
use ::std::collections::HashMap;
use ::std::fs;
use ::std::path::{Path, PathBuf};

#[derive(Debug, Deserialize, Default)]
pub struct TheiaRootPluginConfig {
    /// Path to the plugin directory
    pub path: PathBuf,

    /// Plugin configuration data
    #[serde(default, rename = "config")]
    pub cfgdata: HashMap<String, Value>,
}

#[derive(Debug, Deserialize, Default)]
pub struct TheiaConfig {
    /// List of command prefixes
    pub prefixes: Vec<String>,

    /// List of Discord user IDs that can perform administration commands
    #[serde(default)]
    pub admin_users: Vec<String>,

    /// Total shard count.
    pub shard_count: u64,

    /// List of paths to plugins to load
    #[serde(default, rename = "plugins")]
    pub plugin_cfg: HashMap<String, TheiaRootPluginConfig>,
}

impl TheiaConfig {
    pub fn new(config_path: &Path) -> Result<Self, TheiaError> {
        let config_str: String =
            fs::read_to_string(config_path).map_err(|_| TheiaError::ConfigNotFound)?;
        let config: Self =
            ::toml::from_str(&config_str).map_err(|e| TheiaError::ConfigParseError(e))?;

        if config.admin_users.len() < 1 {
            ::tracing::warn!("WARNING: No admin users defined in the configuration!");
        }

        if config.plugin_cfg.len() < 1 {
            ::tracing::warn!("WARNING: No plugins listed! This bot won't do much...");
        }

        Ok(config)
    }
}
