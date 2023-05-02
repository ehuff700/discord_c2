use crate::utils::agent::Agent;
use serde_json::json;
use serenity::builder::CreateApplicationCommand;
use serenity::model::id::ChannelId;

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    // Create the info command. Description must be under 100 characters.
    command
        .name("info")
        .description("Gather information about the agent")
}

pub fn run(channel_id: &ChannelId, agent: Agent) -> String {
    if channel_id == agent.get_command_channel() {
        let data = json!(agent);

        // Format the JSON string with indentation
        let formatted = serde_json::to_string_pretty(&data).unwrap();

        format!("**Agent Info** \n```json\n{}\n```", formatted)
    } else {
        " ".to_string()
    }
}
