use serde_json::json;

use crate::{utils::agent::get_or_create_agent, Context, Error};

/// Retrieves information about the agent
#[poise::command(slash_command)]
pub async fn info(ctx: Context<'_>) -> Result<(), Error> {
	let agent = get_or_create_agent(ctx.serenity_context()).await;

	// Making sure the command was sent in a valid channel, otherwise, defer the interaction.
	if agent.get_command_channel() == &ctx.channel_id() {
		let data = json!(agent);

		// Format the JSON string with indentation
		let mut formatted = serde_json::to_string_pretty(&data).unwrap();
		formatted = format!("Agent Info \n```json\n{}\n```", formatted);

		ctx.say(formatted).await?;
	} else {
		ctx.say("x").await?; // find a better way to do this
	}

	Ok(())
}
