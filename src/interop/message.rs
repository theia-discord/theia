use crate::prelude::*;
use ::serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TheiaDiscordMessage {
    pub message_id: String,
    pub channel_id: String,
    pub author_id: String,
    pub guild_id: Option<String>,
    pub webhook_id: Option<String>,
    pub content: String,
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
        }
    }
}
