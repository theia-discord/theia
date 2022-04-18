use crate::plugin::TheiaPluginConfig;
use crate::prelude::*;
use ::serde::{Deserialize, Serialize};
use ::serde_json::Value;
use ::serenity::model::id::{ChannelId, MessageId};
use ::std::collections::HashMap;
use ::std::str::FromStr;

#[derive(Debug, Serialize, Deserialize)]
pub enum TheiaPluginOutgoingMessage {
    BotInfo {
        prefixes: Vec<String>,
        admin_users: Vec<String>,
        invite_url: Option<String>,
        this_shard: u64,
        total_shards: u64,
        plugins: Vec<String>,
        commands: Vec<(String, String)>,
    },

    PluginConfig {
        #[serde(rename = "name")]
        plugin_name: String,
        config: HashMap<String, Value>,
    },

    CommandInvoke {
        cmd: CommandInvocation,
        message: TheiaDiscordMessage,
    },
}

impl TheiaPluginOutgoingMessage {
    pub async fn bot_info<'x>(ctx: &'x SerenityContext) -> Self {
        let data = ctx.data.read().await;
        let theia = data.get::<TheiaContainer>().unwrap();

        Self::BotInfo {
            prefixes: theia.prefixes(),
            admin_users: theia.config.admin_users.clone(),
            invite_url: None,
            this_shard: ctx.shard_id,
            total_shards: theia.config.shard_count,
            plugins: theia.plugin_names(),
            commands: theia.plugin_command_summaries(),
        }
    }

    pub async fn plugin_config<'x>(cfg: &'x TheiaPluginConfig) -> Self {
        Self::PluginConfig {
            plugin_name: cfg.name.clone(),
            config: cfg.cfgdata.clone(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[non_exhaustive]
pub enum TheiaPluginIncomingMessage {
    SendMessage {
        channel_id: String,
        in_reply_to: Option<String>,
        content: String,
    },
}

impl TheiaPluginIncomingMessage {
    pub async fn process<'c>(&self, ctx: &'c SerenityContext) -> Result<(), TheiaError> {
        trace!("Processing incoming plugin message: {:?}", self);
        match self {
            Self::SendMessage {
                channel_id,
                in_reply_to,
                content,
                ..
            } => {
                let channel_id = ChannelId::from(u64::from_str(channel_id)?);
                let mut message_id: Option<MessageId> = None;
                if let Some(mid) = in_reply_to {
                    message_id = Some(MessageId::from(u64::from_str(mid)?));
                }

                channel_id
                    .send_message(&ctx.http, |m| {
                        if let Some(mid) = message_id {
                            m.reference_message((channel_id, mid));
                        }

                        m.content(content);
                        m
                    })
                    .await?;

                Ok(())
            }
        }
    }
}
