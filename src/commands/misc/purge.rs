use poise::serenity_prelude::MessageId;

use crate::{Context, Error};

/// Runs the message purging process on the given channel.
#[poise::command(slash_command)]
pub async fn purge(ctx: Context<'_>) -> Result<(), Error> {
	ctx.defer().await?;

	let serenity_context = ctx.serenity_context();
	let channel_id = ctx.channel_id();

	loop {
		// Your async code goes here
		let messages = match channel_id.messages(&serenity_context.http, |retriever| retriever.limit(30)).await {
			Ok(messages) if messages.is_empty() => {
				break;
			},
			Ok(messages) => messages,
			Err(e) => {
				ctx.say(format!("Ran into an error when fetching messages: {}", e)).await?;
				return Err(Error::from(e)); // Convert the Serenity error into your custom error type
			},
		};

		let message_ids: Vec<MessageId> = messages.iter().map(|msg| msg.id).collect();
		channel_id.delete_messages(&serenity_context.http, &message_ids).await?;
	}

	Ok(())
}
