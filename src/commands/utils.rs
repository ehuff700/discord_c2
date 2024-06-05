use poise::serenity_prelude::futures::StreamExt;
use tracing::error;

use crate::{say, RuscordContext, RuscordError};

/// Displays this help message.
#[poise::command(prefix_command, slash_command)]
pub async fn help(
	ctx: RuscordContext<'_>,
	#[description = "Specific command to show help about"] command: Option<String>,
) -> Result<(), RuscordError> {
	let config = poise::builtins::PrettyHelpConfiguration {
		show_subcommands: true,
		ephemeral: true,
		extra_text_at_bottom: "\
Type !help command for more info on a command.",
		..Default::default()
	};
	poise::builtins::pretty_help(ctx, command.as_deref(), config).await?;
	Ok(())
}

/// Clears the current channel of all messages.
#[poise::command(prefix_command, slash_command)]
pub async fn clear(
	ctx: RuscordContext<'_>,
	#[description = "Amount of messages to delete. If left empty, it will recreate the channel."] count: Option<i32>,
) -> Result<(), RuscordError> {
	let channel_id = ctx.channel_id();
	let messages = channel_id.messages_iter(&ctx).boxed();

	match count {
		Some(c) => {
			let mut stream = messages.take(c as usize);
			let mut counter = 0;
			while let Some(message_result) = stream.next().await {
				match message_result {
					Ok(message) => {
						message.delete(&ctx.http()).await?;
						counter += 1;
					},
					Err(error) => error!("Error retrieving message: {}", error),
				}
			}
			say!(&ctx, "Successfully deleted {} messages", counter);
		},
		None => {
			let mut guard = ctx.data().config.lock().await;
			guard.command_channel.delete(ctx.http()).await?;
			guard.reset_command_channel(ctx.serenity_context()).await?;
			guard.command_channel.say(ctx.http(), "Messages successfully cleared").await?;
		},
	}
	Ok(())
}
