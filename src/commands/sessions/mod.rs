use serenity::client::Context;

pub mod exit;
pub mod session;
pub mod download;
pub mod upload;

// Helper function to pull the command ID from a command name
async fn get_command_id_by_name(ctx: &Context, command_name: &str) -> Option<u64> {
    // Get the list of global application commands
    let commands = ctx.http.get_global_application_commands().await.ok()?;

    // Find the command with the matching name and return its ID
    for command in commands {
        if command.name == command_name {
            return Some(u64::from(command.id));
        }
    }

    None
}
