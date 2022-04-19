use crate::util::runnable::RunnableCommand;
use ::serde::Deserialize;
use ::serde_json::Value;
use ::std::collections::HashMap;

/// Configuration for a plugin.
#[derive(Debug, Deserialize)]
pub struct TheiaPluginConfig {
    /// Plugin name.
    pub name: String,

    /// Executable to run for this plugin.
    pub run: RunnableCommand,

    /// Whether to pass through help invocations directly to the commands.
    ///
    /// This will apply to all commands, unless overridden in an individual
    /// command description.
    #[serde(default, rename = "help-passthrough")]
    pub help_passthrough: bool,

    /// Plugin configuration data.
    ///
    /// This is sent in a `PluginConfig` event to the plugin on initialization.
    #[serde(default, rename = "config")]
    pub cfgdata: HashMap<String, Value>,

    /// List of commands known by this plugin.
    #[serde(rename = "command")]
    pub commands: Vec<TheiaPluginCommandConfig>,
}

impl TheiaPluginConfig {
    pub fn handle_help<T: AsRef<str>>(&self, cmd_name: T) -> bool {
        if let Some(command) = self.commands.iter().find(|e| e.name == cmd_name.as_ref()) {
            if let Some(passthru) = command.help_passthrough {
                return !passthru;
            }
        }

        !self.help_passthrough
    }
}

/// Configuration for an individual plugin-based command.
#[derive(Debug, Deserialize)]
pub struct TheiaPluginCommandConfig {
    /// Name of this command.
    pub name: String,

    /// Aliases for this command.
    #[serde(default)]
    pub aliases: Vec<String>,

    /// Whether to pass through help invocations directly to this command.
    #[serde(default, rename = "help-passthrough")]
    pub help_passthrough: Option<bool>,

    /// Help text for this command.
    #[serde(default)]
    pub help: String,

    /// Whether this command is hidden in global command listings.
    #[serde(default)]
    pub hidden: bool,
}
