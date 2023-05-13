use crate::{errors::DiscordC2Error, discord_utils::bot_functions::{send_ephemeral_response, send_channel_message}};

use serenity::{
    builder::CreateApplicationCommand,
    model::{
        application::interaction::application_command::ApplicationCommandInteraction,
        id::{ChannelId, MessageId},
    },
    prelude::*,
};

use tokio::task::spawn;

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

/// Runs the message purging process on the given channel.
///
/// This function asynchronously purges messages in the specified channel by continuously
/// fetching and deleting messages until no more messages are returned.
/// It returns a message indicating the result of the purging process.
///
/// # Arguments
///
/// * `ctx` - The Serenity context.
/// * `channel_id` - The ID of the channel to purge messages from.
///
/// # Returns
///
/// A `String` indicating the result of the purging process.
pub async fn run(ctx: &Context, channel_id: &ChannelId) -> String {

    // Clone `ctx` and `channel_id`
    let cloned_ctx = ctx.clone();
    let cloned_channel_id = *channel_id;

    // Spawn a new task (not a thread)
    spawn(async move {
        loop {
            // Your async code goes here
            let messages = match cloned_channel_id
                .messages(&cloned_ctx.http, |retriever| retriever.limit(100))
                .await
            {
                Ok(messages) if messages.is_empty() => break,
                Ok(messages) => messages,
                Err(e) => {
                    send_channel_message(&cloned_ctx, cloned_channel_id, format!("Ran into an error when fetching messages: {}", e)).await.unwrap_or_default();
                    return
                },
            };

            let message_ids: Vec<MessageId> = messages.iter().map(|msg| msg.id).collect();
            if let Err(e) = cloned_channel_id.delete_messages(&cloned_ctx.http, &message_ids).await {
                send_channel_message(&cloned_ctx, cloned_channel_id, format!("Ran into an error when attempting to delete messages: {}", e)).await.unwrap_or_default();
                return;
            }
        }
    });

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
    send_ephemeral_response(ctx, command, &message_content).await?;
    Ok(())
}
