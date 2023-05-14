use crate::utils::agent::Agent;
use serde_json::json;
use serenity::{builder::CreateApplicationCommand, model::id::ChannelId};

/// Registers the `info` command.
///
/// # Arguments
///
/// * `command` - A mutable reference to the `CreateApplicationCommand` builder.
///
/// # Returns
///
/// A mutable reference to the modified `CreateApplicationCommand` builder.
pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    // Create the info command. Description must be under 100 characters.
    command
        .name("info")
        .description("Gather information about the agent")
}

/// Runs the `info` command to gather information about the agent.
///
/// # Arguments
///
/// * `channel_id` - The ID of the channel in which the command was issued.
/// * `agent` - An instance of the `Agent` struct.
///
/// # Returns
///
/// A string containing the formatted JSON representation of the `agent`.
pub fn run(channel_id: &ChannelId, agent: Agent) -> String {
    if channel_id == agent.get_command_channel() {
        let data = json!(agent);

        // Format the JSON string with indentation
        let formatted = serde_json::to_string_pretty(&data).unwrap();
        format!("Agent Info \n```json\n{}\n```", formatted)
    } else {
        " ".to_string()
    }
}
