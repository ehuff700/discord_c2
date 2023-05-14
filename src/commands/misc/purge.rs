use crate::{
    discord_utils::bot_functions::{send_edit_response, send_ephemeral_response},
    errors::DiscordC2Error,
};

use serenity::{
    builder::CreateApplicationCommand,
    model::{
        application::interaction::application_command::ApplicationCommandInteraction,
        id::{ChannelId, MessageId},
    },
    prelude::*,
};
use tracing::error;

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
pub async fn run(
    ctx: Context,
    channel_id: ChannelId,
    command: ApplicationCommandInteraction,
) -> String {
    // Spawn a new task to loop over messages and delete them // TODO: Make this method better get rid of unwrap or def
    tokio::spawn(async move {
        loop {
            // Your async code goes here
            let messages = match channel_id
                .messages(&ctx.http, |retriever| retriever.limit(30))
                .await
            {
                Ok(messages) if messages.is_empty() => break,
                Ok(messages) => messages,
                Err(e) => {
                    match send_edit_response(
                        &ctx,
                        &command,
                        format!("Ran into an error when fetching messages: {}", e),
                    )
                    .await
                    {
                        Ok(_) => return,
                        Err(why) => {
                            error!(
                                "Error sending edit response: {}. Original error: {}",
                                why, e
                            );
                        }
                    };
                    return;
                }
            };

            let message_ids: Vec<MessageId> = messages.iter().map(|msg| msg.id).collect();
            if let Err(e) = channel_id.delete_messages(&ctx.http, &message_ids).await {
                match send_edit_response(
                    &ctx,
                    &command,
                    format!("Ran into an error when fetching messages: {}", e),
                )
                .await
                {
                    Ok(_) => return,
                    Err(why) => {
                        error!(
                            "Error sending edit response: {}. Original error: {}",
                            why, e
                        );
                    }
                };
            }
        }
        match send_edit_response(&ctx, &command, "Messages successfully purged!").await {
            Ok(_) => (),
            Err(why) => {
                error!("Error sending edit response: {}", why);
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
pub async fn purge_handler(
    ctx: &Context,
    command: &ApplicationCommandInteraction,
) -> Result<(), DiscordC2Error> {
    let message = send_ephemeral_response(ctx, command, "Purging....", None).await?;
    run(ctx.to_owned(), command.channel_id, message).await;

    Ok(())
}
