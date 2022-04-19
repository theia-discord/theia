use crate::prelude::*;
use ::serde::{Deserialize, Serialize};
use ::serenity::model::id::{ChannelId, MessageId};
use ::std::str::FromStr;

pub mod cmdhooks;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TheiaDiscordMessage {
    pub message_id: String,
    pub channel_id: String,
    pub author_id: String,
    pub guild_id: Option<String>,
    pub webhook_id: Option<String>,
    pub content: String,

    pub command_invocation: Option<CommandInvocation>,
    pub message_proxy: Option<()>,
}

impl TheiaDiscordMessage {
    pub async fn reply<H: AsRef<serenity::http::Http>>(
        &self,
        http: H,
        content: impl ::std::fmt::Display,
    ) -> Result<SerenityDiscordMessage, TheiaError> {
        let channel_id = ChannelId::from(u64::from_str(&self.channel_id)?);
        let message_id = MessageId::from(u64::from_str(&self.message_id)?);

        let msg = channel_id
            .send_message(http, |m| {
                m.reference_message((channel_id, message_id));

                m.content(content);
                m
            })
            .await?;

        Ok(msg)
    }

    pub async fn parse_as_command<'ctx>(
        &self,
        ctx: &'ctx SerenityContext,
    ) -> Result<Option<Self>, TheiaError> {
        let mut msg = self.clone();

        macro_rules! run_hook {
            ($hookfn:path) => {
                if let Some(nmsg) = ($hookfn)(ctx, msg).await? {
                    msg = nmsg;
                } else {
                    return Ok(None);
                }
            };
        }

        run_hook!(cmdhooks::parse_command);
        Ok(Some(msg))
    }
}

impl From<SerenityDiscordMessage> for TheiaDiscordMessage {
    fn from(msg: SerenityDiscordMessage) -> TheiaDiscordMessage {
        TheiaDiscordMessage {
            message_id: format!("{}", msg.id.0),
            channel_id: format!("{}", msg.channel_id.0),
            author_id: format!("{}", msg.author.id.0),
            guild_id: msg.guild_id.map(|e| format!("{}", e.0)),
            webhook_id: msg.webhook_id.map(|e| format!("{}", e.0)),
            content: String::from(msg.content),
            command_invocation: None,
            message_proxy: None,
        }
    }
}
