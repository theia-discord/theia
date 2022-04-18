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
        msg: &SerenityDiscordMessage,
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
    async fn ready(&self, ctx: SerenityContext, ready: Ready) {
        let data = ctx.data.read().await;
        let theia = data.get::<TheiaContainer>().unwrap();

        let mut status: Vec<String> = vec![format!("{}help", theia.prefix())];

        if let Some(shard) = ready.shard {
            info!("Shard {} of {} ready", shard[0], shard[1]);
            status.push(format!("shard {}", shard[0]));
        }

        let status = status.join(" \u{2022} ").to_string();
        ctx.shard
            .set_presence(Some(Activity::listening(status)), OnlineStatus::Online);
    }

    async fn message(&self, ctx: SerenityContext, msg: SerenityDiscordMessage) {
        let data = ctx.data.read().await;
        let theia = data.get::<TheiaContainer>().unwrap();

        let tmsg = TheiaDiscordMessage::from(msg.clone());
        if let Some(mut cmd) = CommandInvocation::parse(&theia.prefixes(), &msg.content) {
            // msg.reply(&ctx.http, format!("```\n{:#?}\n```", cmd)).await.unwrap();
            if let Some(plugin) = theia.plugin_with_command(&cmd.command) {
                if let Some(plugin_cmd) = plugin.config.commands.iter().find(|c| {
                    &c.name == &cmd.command || c.aliases.iter().any(|a| a == &cmd.command)
                }) {
                    cmd.command = plugin_cmd.name.clone();
                }

                if plugin.config.handle_help(&cmd.command) && cmd.help_requested() {
                    let mut help = format!(
                        "\u{274c} No help available for `{prefix}{0}`",
                        &cmd.command,
                        prefix = theia.prefix()
                    );

                    if let Some(cmdcfg) = plugin.command_config(&cmd.command) {
                        let (mut help_s, help_details) =
                            parse_command_help(theia.prefix(), &cmdcfg.name, &cmdcfg.help);

                        if let Some(details) = help_details {
                            help_s = format!("{}\n\n{}", help_s, details);
                        }

                        help = help_s;
                    }

                    if let Err(se) = msg.reply(&ctx.http, help).await {
                        self.handle_err_reply(&ctx, &msg, TheiaError::from(se))
                            .await;
                    }
                } else {
                    if let Err(why) = plugin.invoke_command(&ctx, &cmd, &tmsg).await {
                        self.handle_err_reply(&ctx, &msg, why).await;
                    }
                }
            } else {
                if let Err(se) = msg
                    .reply(
                        &ctx.http,
                        format!(
                            "\u{274c} Command `{prefix}{0}` not found.",
                            &cmd.command,
                            prefix = theia.prefix()
                        ),
                    )
                    .await
                {
                    self.handle_err_reply(&ctx, &msg, TheiaError::from(se))
                        .await;
                }
            }
        }
    }
}
