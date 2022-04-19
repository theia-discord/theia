use crate::prelude::*;

pub async fn parse_command<'ctx>(
    ctx: &'ctx SerenityContext,
    mut msg: TheiaDiscordMessage,
) -> Result<Option<TheiaDiscordMessage>, TheiaError> {
    let data = ctx.data.read().await;
    let theia = data.get::<TheiaContainer>().unwrap();

    if let Some(mut cmd) = CommandInvocation::parse(&theia.prefixes(), &msg.content) {
        if let Some(plugin) = theia.plugin_with_command(&cmd.command) {
            if let Some(plugin_cmd) =
                plugin.config.commands.iter().find(|c| {
                    &c.name == &cmd.command || c.aliases.iter().any(|a| a == &cmd.command)
                })
            {
                cmd.command = plugin_cmd.name.clone();
            }
        }

        cmd.generate_invoke_id();
        msg.command_invocation = Some(cmd);
        Ok(Some(msg))
    } else {
        Ok(None)
    }
}
