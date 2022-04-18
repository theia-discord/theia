//! The Theia plugin architecture.

use crate::error::{TheiaError, TheiaPluginLoadError, TheiaPluginRunError};
use crate::prelude::*;

use ::serde_json::Value;
use ::std::collections::HashMap;
use ::std::fs;
use ::std::path::{Path, PathBuf};
use ::std::process::Stdio;
use ::tokio::io::AsyncWriteExt;

pub mod comms;
use self::comms::*;

mod config;
pub use self::config::*;

/// A plugin.
#[derive(Debug)]
pub struct TheiaPlugin {
    pub path: PathBuf,
    pub config: TheiaPluginConfig,
    pub cfgdata: HashMap<String, Value>,
}

impl TheiaPlugin {
    pub fn new(path: &Path) -> Result<Self, TheiaError> {
        let path: PathBuf = PathBuf::from(path);
        let cfg_path: PathBuf = {
            let mut cfg_path = path.clone();
            cfg_path.push("theia-plugin.toml");
            cfg_path
        };

        let cfg_str: String = fs::read_to_string(&cfg_path).map_err(|_| {
            TheiaError::PluginLoad(
                String::from(path.to_string_lossy()),
                TheiaPluginLoadError::NotFound,
            )
        })?;

        let config: TheiaPluginConfig = ::toml::from_str(&cfg_str).map_err(|e| {
            TheiaError::PluginLoad(
                String::from(path.to_string_lossy()),
                TheiaPluginLoadError::ConfigParseError(e),
            )
        })?;

        Ok(Self {
            path,
            config,
            cfgdata: Default::default(),
        })
    }

    pub async fn configure(&mut self, data: &HashMap<String, Value>) -> Result<(), TheiaError> {
        self.cfgdata = data.clone();
        Ok(())
    }
}

impl<'a> TheiaPlugin {
    pub fn name(&'a self) -> &'a str {
        &self.config.name
    }

    pub fn command_config<C: AsRef<str>>(
        &'a self,
        cmd_name: C,
    ) -> Option<&'a TheiaPluginCommandConfig> {
        self.config
            .commands
            .iter()
            .find(|p| p.name == cmd_name.as_ref())
    }

    pub async fn invoke(
        &'a self,
        msgs: &[TheiaPluginOutgoingMessage],
    ) -> Result<Vec<TheiaPluginIncomingMessage>, TheiaError> {
        let to_write: String = {
            let mut to_write = Vec::new();
            for msg in msgs.iter() {
                to_write.push(::serde_json::to_string(msg)?);
            }

            format!("{}\n", to_write.join("\n"))
        };

        let mut child = self
            .config
            .run
            .as_tokio_command()
            .current_dir(self.path.clone())
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()?;

        // write to child stdin
        if let Some(mut stdin) = child.stdin.take() {
            stdin.write_all(to_write.as_bytes()).await?;
        }

        // grab output
        let output = child.wait_with_output().await?;
        if !output.status.success() {
            if let Some(code) = output.status.code() {
                return Err(TheiaError::PluginRun(
                    self.config.name.clone(),
                    TheiaPluginRunError::ExitStatus(code),
                ));
            } else {
                return Err(TheiaError::PluginRun(
                    self.config.name.clone(),
                    TheiaPluginRunError::Terminated,
                ));
            }
        }

        // convert output to string
        let output: Vec<String> = String::from_utf8(output.stdout)?
            .split("\n")
            .map(|x| x.trim())
            .filter(|x| &"" != x)
            .map(String::from)
            .collect();

        let mut responses: Vec<TheiaPluginIncomingMessage> = Vec::new();
        for resp in output.iter() {
            let response: TheiaPluginIncomingMessage = ::serde_json::from_str(&resp)?;
            responses.push(response);
        }

        Ok(responses)
    }

    pub async fn invoke_command<'x>(
        &'a self,
        ctx: &'x SerenityContext,
        cmd: &'x CommandInvocation,
        msg: &'x TheiaDiscordMessage,
    ) -> Result<(), TheiaError> {
        let mut responses = self
            .invoke(&[
                TheiaPluginOutgoingMessage::bot_info(&ctx).await,
                TheiaPluginOutgoingMessage::plugin_config(&self).await,
                TheiaPluginOutgoingMessage::CommandInvoke {
                    cmd: cmd.clone(),
                    message: msg.clone(),
                },
            ])
            .await?;

        for resp in responses.iter_mut() {
            resp.process(&ctx).await?;
        }

        Ok(())
    }
}
