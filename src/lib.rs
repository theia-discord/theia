//! The core of Theia.

pub mod config;
pub mod error;
pub mod event;
pub mod interop;
pub mod parser;
pub mod plugin;
pub mod typemap;
pub mod util;

pub mod prelude;
use self::prelude::*;

use self::event::TheiaEventHandler;
use ::serenity::client::Client as SerenityClient;
use ::std::path::{Path, PathBuf};
use ::std::str::FromStr;
use ::std::sync::Arc;

pub struct Theia {
    pub config_path: PathBuf,
    pub config: TheiaConfig,
    plugins: Vec<TheiaPlugin>,
}

impl Theia {
    pub fn new(config_path: &Path) -> Result<Self, TheiaError> {
        let config_path = PathBuf::from(config_path);

        Ok(Self {
            config_path,
            config: Default::default(),
            plugins: Vec::new(),
        })
    }

    pub async fn reload(&mut self) -> Result<(), TheiaError> {
        info!("Reloading Theia configuration...");

        // Load config
        self.config = TheiaConfig::new(&self.config_path)?;

        // Load plugins
        self.plugins = Vec::new();
        for (_plugin_name, plugin_cfg) in self.config.plugin_cfg.iter() {
            let mut plugin = TheiaPlugin::new(&plugin_cfg.path)?;
            info!(
                "Loaded plugin {0:?} (from {path:?})",
                plugin.config.name,
                path = &plugin.path,
            );

            plugin.configure(&plugin_cfg.cfgdata).await?;
            self.plugins.push(plugin);
        }

        Ok(())
    }

    pub async fn run(self) -> Result<(), TheiaError> {
        let token = ::std::env::var("DISCORD_TOKEN").map_err(|_| TheiaError::NoDiscordToken)?;

        let shard_count = self.config.shard_count;
        let shard_range: [u64; 2] = [
            u64::from_str(&::std::env::var("THEIA_SHARDRANGE_START").unwrap_or(String::new()))
                .unwrap_or(0),
            u64::from_str(&::std::env::var("THEIA_SHARDRANGE_END").unwrap_or(String::new()))
                .unwrap_or(shard_count - 1),
        ];

        let mut serenity_client = SerenityClient::builder(&token)
            .event_handler(TheiaEventHandler)
            .await
            .map_err(|se| TheiaError::SerenityError(se))?;

        {
            let mut data = serenity_client.data.write().await;

            data.insert::<TheiaContainer>(Arc::new(self));
            data.insert::<ShardManagerContainer>(Arc::clone(&serenity_client.shard_manager));
        }

        if let Err(se) = serenity_client
            .start_shard_range(shard_range, shard_count)
            .await
        {
            Err(TheiaError::SerenityError(se))?
        }

        Ok(())
    }
}

impl<'a> Theia {
    pub fn prefix(&'a self) -> &'a str {
        &self.config.prefixes[0]
    }

    pub fn prefixes(&'a self) -> Vec<String> {
        self.config.prefixes.iter().map(String::clone).collect()
    }

    pub fn plugin<T: AsRef<str>>(&'a self, name: T) -> Option<&'a TheiaPlugin> {
        self.plugins.iter().find(|p| p.config.name == name.as_ref())
    }

    pub fn plugin_with_command<T: AsRef<str>>(&'a self, cmd_name: T) -> Option<&'a TheiaPlugin> {
        self.plugins.iter().find(|p| {
            p.config.commands.iter().any(|c| {
                c.name == cmd_name.as_ref() || c.aliases.iter().any(|a| a == cmd_name.as_ref())
            })
        })
    }

    pub fn plugin_names(&'a self) -> Vec<String> {
        self.plugins.iter().map(|p| p.config.name.clone()).collect()
    }

    pub fn plugin_command_summaries(&'a self) -> Vec<(String, String)> {
        self.plugins
            .iter()
            .map(|p| {
                p.config
                    .commands
                    .iter()
                    .filter(|cmdcfg| !cmdcfg.hidden)
                    .map(|cmdcfg| {
                        let (help_summary, _) =
                            parse_command_help(self.prefix(), &cmdcfg.name, &cmdcfg.help);
                        (String::from(&cmdcfg.name), help_summary)
                    })
            })
            .flatten()
            .collect()
    }
}
