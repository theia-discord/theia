use crate::prelude::*;
use ::serenity::client::EventHandler;
use ::serenity::model::gateway::{Activity, Ready};
use ::serenity::model::user::OnlineStatus;

/// The Serenity event handler
pub struct TheiaEventHandler;

impl TheiaEventHandler {
    pub async fn handle_err_reply(
        &self,
        ctx: &SerenityContext,
        msg: &TheiaDiscordMessage,
        why: TheiaError,
    ) {
        let errcode = self.handle_err(why).await;

        // Ignore any errors while sending the error message
        let _ = msg
            .reply(
                &ctx.http,
                format!("\u{274c} An error occurred: `{0}`", errcode),
            )
            .await;
    }

    pub async fn handle_err(&self, why: TheiaError) -> String {
        error!("Unhandled TheiaError: {:?}", why);
        return String::from("00000000000000000000000000000000");
    }
}

#[async_trait]
impl EventHandler for TheiaEventHandler {
    async fn ready(&self, ctx: SerenityContext, _ready: Ready) {
        let data = ctx.data.read().await;
        let theia = data.get::<TheiaContainer>().unwrap();

        let status: Vec<String> = vec![
            format!("{}help", theia.prefix()),
            format!("shard {}", ctx.shard_id),
        ];

        let status_str = status.join(" \u{2022} ").to_string();
        ctx.set_presence(Some(Activity::listening(status_str)), OnlineStatus::Online)
            .await;
    }

    async fn message(&self, ctx: SerenityContext, orig_msg: SerenityDiscordMessage) {
        let msg = TheiaDiscordMessage::from(orig_msg.clone());
        let cmd_msg = match msg.parse_as_command(&ctx).await {
            Err(why) => {
                self.handle_err_reply(&ctx, &msg, why).await;
                return;
            }

            Ok(None) => {
                return;
            }

            Ok(Some(cmd_msg)) => cmd_msg,
        };

        if let Err(why) = invoke_command(&ctx, &cmd_msg).await {
            self.handle_err_reply(&ctx, &cmd_msg, why).await;
        }
    }
}
