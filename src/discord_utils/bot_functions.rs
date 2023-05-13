
use serenity::{
    client::Context,
    model::prelude::{ChannelId, interaction::{application_command::ApplicationCommandInteraction, InteractionResponseType}}
};

use crate::errors::DiscordC2Error;
use tracing::error;

/// Sends a formatted message to a specified channel.
///
/// # Arguments
///
/// * `ctx` - The context object containing information about the current state of the bot.
/// * `channel_id` - The ID of the channel where the message should be sent.
/// * `message` - The content of the message to be sent.
/// * `language_format` - The language format to be used for code blocks in the message.
///
/// # Returns
///
/// * `Result<(), String>` - A result indicating success or failure. Returns `Ok(())` if the message was sent successfully, or `Err(String)` with an error message if an error occurred.
///
/// # Example
///
/// ```rust
/// use serenity::model::id::ChannelId;
/// use serenity::prelude::Context;
///
/// # async fn example() {
/// # let ctx = Context::new();
/// # let channel_id = ChannelId::new(1234567890);
/// # let message = "Hello, world!";
/// # let language_format = "rust";
/// if let Err(error) = send_message(&ctx, channel_id, message, language_format).await {
///     println!("Error sending message: {}", error);
/// }
/// # }
/// ```
pub async fn send_code_message<T: AsRef<str>>(ctx: &Context, channel_id: ChannelId, message: T, language_format: &str) -> Result<(), String> {

    let output_chunks = message.as_ref().split('\n');
    let fence = format!("```{}\n", language_format);
    let fence_length = fence.len() + 3; // 3 for the closing fence
    let mut buffer = fence.clone();

    for line in output_chunks {
        if buffer.len() + line.len() + 1 + fence_length > 2000 {
            buffer.push_str("```");
            if let Err(why) = channel_id.say(&ctx.http, &buffer).await {
                println!("Error sending message: {:?}", why);
                return Err(format!("Error sending message: {:?}", why));
            } 
            buffer = fence.clone();
        }
        buffer.push_str(line);
        buffer.push('\n');
    }

    if !buffer.is_empty() {
        buffer.push_str("```");
        if let Err(why) = channel_id.say(&ctx.http, &buffer).await {
            println!("Error sending message: {:?}", why);
            return Err(format!("Error sending message: {:?}", why));
        }
    }

    Ok(())
}

/// Sends a message to a Discord channel using the provided context.
///
/// # Arguments
///
/// * `ctx` - The context object representing the current bot state.
/// * `channel_id` - The ID of the channel where the message will be sent.
/// * `message` - The content of the message to be sent.
///
/// # Returns
///
/// Returns `Ok(())` if the message was sent successfully. If an error occurs during message
/// sending, it returns a `DiscordC2Error` containing the details of the error.
///
/// # Examples
///
/// ```rust
/// # use serenity::model::id::ChannelId;
/// # use serenity::client::Context;
/// # use your_crate_name::send_channel_message;
/// #
/// # async fn example(ctx: &Context, channel_id: ChannelId) {
///     if let Err(err) = send_channel_message(ctx, channel_id, "Hello, world!").await {
///         println!("Failed to send message: {:?}", err);
///     }
/// # }
/// ```
pub async fn send_channel_message<T: AsRef<str>>(ctx: &Context, channel_id: ChannelId, message: T) -> Result<(), DiscordC2Error> {
    if let Err(why) = channel_id.say(&ctx.http, message.as_ref()).await {
        error!("Error sending message: {:?}", why);
        return Err(DiscordC2Error::from(why))
    }
    Ok(())
}

/// Creates an ephemeral response to an application command interaction.
///
/// This function sends a response to the interaction in the form of an ephemeral message,
/// meaning only the user who issued the command can see the response.
///
/// # Arguments
///
/// * `ctx` - The context object containing information about the current state of the bot.
/// * `command` - The application command interaction object representing the received command.
/// * `content` - The content of the ephemeral message response.
///
/// # Returns
///
/// Returns `Ok(())` if the response is sent successfully, otherwise returns a `DiscordC2Error`.
///
/// # Examples
///
/// ```
/// use discord_c2::DiscordC2Error;
///
/// async fn handle_command(ctx: &Context, command: &ApplicationCommandInteraction) -> Result<(), DiscordC2Error> {
///     let content = "Hello, this is an ephemeral response!";
///     create_ephemeral_response(ctx, command, content).await?;
///     Ok(())
/// }
/// ```
pub async fn send_ephemeral_response<T: AsRef<str>>(
    ctx: &Context,
    command: &ApplicationCommandInteraction,
    content: T,
) -> Result<(), DiscordC2Error> {
    command
        .create_interaction_response(&ctx.http, |response| {
            response
                .kind(InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|message| message.content(content.as_ref()).ephemeral(true))
        })
        .await?;
    Ok(())
}