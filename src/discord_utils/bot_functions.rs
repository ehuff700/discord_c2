use serenity::{
    client::Context,
    model::prelude::{
        interaction::{
            application_command::ApplicationCommandInteraction,
            InteractionResponseType,
        },
        AttachmentType,
        ChannelId,
    },
};
use tracing::error;

use crate::errors::DiscordC2Error;

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
pub async fn send_code_message<'a, T: AsRef<str>>(
    ctx: &'a Context,
    channel_id: ChannelId,
    message: T,
    language_format: &str
) -> Result<(), String> {
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
/// # async fn example(ctx: &'a Context, channel_id: ChannelId) {
///     if let Err(err) = send_channel_message(ctx, channel_id, "Hello, world!").await {
///         println!("Failed to send message: {:?}", err);
///     }
/// # }
/// ```
pub async fn send_channel_message<T: AsRef<str>>(
    ctx: &Context,
    channel_id: ChannelId,
    message: T
) -> Result<(), DiscordC2Error> {
    if let Err(why) = channel_id.say(&ctx.http, message.as_ref()).await {
        error!("Error sending message: {:?}", why);
        return Err(DiscordC2Error::from(why));
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
/// async fn handle_command(ctx: &'a Context, command: &'a ApplicationCommandInteraction) -> Result<(), DiscordC2Error> {
///     let content = "Hello, this is an ephemeral response!";
///     create_ephemeral_response(ctx, command, content).await?;
///     Ok(())
/// }
/// ```
pub async fn send_ephemeral_response<'a, T: AsRef<str>>(
    ctx: &'a Context,
    command: &'a ApplicationCommandInteraction,
    content: T,
    attachment: Option<AttachmentType<'static>>
) -> Result<ApplicationCommandInteraction, DiscordC2Error> {
    command.create_interaction_response(&ctx.http, |response| {
        response
            .kind(InteractionResponseType::ChannelMessageWithSource)
            .interaction_response_data(|message| {
                message.content(content.as_ref()).ephemeral(true);
                if let Some(attachment) = attachment {
                    message.add_file(attachment);
                }
                message
            })
    }).await?;
    Ok(command.to_owned())
}
/// Sends an interaction response to Discord.
///
/// This function sends a response to an application command interaction in Discord. It takes a context object, the command interaction itself,
/// the content of the response (which can be a string or anything that can be converted to a string), and an optional attachment (file) to include
/// in the response. The function returns a `Result` indicating whether the response was successfully sent or an error occurred.
///
/// # Arguments
///
/// * `ctx` - A reference to the context object that represents the bot's state and connection to Discord.
/// * `command` - A reference to the application command interaction that triggered the response.
/// * `content` - The content of the response, which can be any type that can be converted to a string.
/// * `attachment` - An optional attachment (file) to include in the response. Pass `None` if no attachment is needed.
///
/// # Returns
///
/// A `Result` containing either the updated `ApplicationCommandInteraction` object on success or a `DiscordC2Error` if an error occurred.
///
/// # Examples
///
/// ```rust
/// use serenity::{
///     model::interactions::ApplicationCommandInteraction,
///     http::AttachmentType,
/// };
/// use serenity::client::Context;
///
/// async fn example_usage(ctx: &Context, command: &ApplicationCommandInteraction) {
///     let content = "Hello, world!";
///     let attachment = None;
///
///     if let Err(err) = send_interaction_response(ctx, command, content, attachment).await {
///         println!("Failed to send interaction response: {:?}", err);
///     }
/// }
/// ```
pub async fn send_interaction_response<'a, T>(
    ctx: &'a Context,
    command: &'a ApplicationCommandInteraction,
    content: T,
    attachment: Option<AttachmentType<'a>>
) -> Result<ApplicationCommandInteraction, DiscordC2Error>
    where T: AsRef<str> + 'a
{
    command.create_interaction_response(&ctx.http, |response| {
        response
            .kind(InteractionResponseType::ChannelMessageWithSource)
            .interaction_response_data(|message| {
                message.content(content.as_ref());

                if let Some(attachment) = attachment {
                    message.add_file(attachment);
                }

                message
            })
    }).await?;
    Ok(command.to_owned())
}

/// Send a follow-up response to an application command interaction.
///
/// This asynchronous function creates a follow-up message for a given interaction.
///
/// # Arguments
///
/// * `ctx` - A reference to the context object.
/// * `command` - A reference to the application command interaction object.
/// * `content` - The content of the follow-up message.
///
/// # Returns
///
/// A result containing an application command interaction object, or a DiscordC2Error in case of failure.
///
/// # Example
///
/// ```rust
/// let response = send_follow_up_response(&ctx, &command, "Follow up message content").await?;
/// ```
pub async fn send_follow_up_response<'a, T: AsRef<str>>(
    ctx: &'a Context,
    command: &'a ApplicationCommandInteraction,
    content: T,
    attachment: Option<AttachmentType<'static>>
) -> Result<ApplicationCommandInteraction, DiscordC2Error> {
    command.create_followup_message(&ctx.http, |message| {
        message.content(content.as_ref());
        if let Some(attachment) = attachment {
            message.add_file(attachment);
        }
        message
    }).await?;

    Ok(command.to_owned())
}

/// Edit the original response to an application command interaction.
///
/// This asynchronous function edits the original response message for a given interaction.
///
/// # Arguments
///
/// * `ctx` - A reference to the context object.
/// * `command` - A reference to the application command interaction object.
/// * `content` - The new content to replace the original response message.
///
/// # Returns
///
/// A result containing an application command interaction object, or a DiscordC2Error in case of failure.
///
/// # Example
///
/// ```rust
/// let response = send_edit_response(&ctx, &command, "New message content").await?;
/// ```
pub async fn send_edit_response<'a, T: AsRef<str>>(
    ctx: &'a Context,
    command: &'a ApplicationCommandInteraction,
    content: T
) -> Result<ApplicationCommandInteraction, DiscordC2Error> {
    command.edit_original_interaction_response(&ctx.http, |message|
        message.content(content.as_ref())
    ).await?;

    Ok(command.to_owned())
}

/// Splits the input string into a vector of strings, each representing a portion of the input that does not exceed the character limit.
///
/// # Arguments
///
/// * `input` - The input string to be split.
/// * `limit` - The maximum number of characters allowed in each portion.
///
/// # Returns
///
/// A vector of strings containing the split portions of the input string.
///
/// # Example
///
/// ```
/// let input = "Lorem ipsum dolor sit amet, consectetur adipiscing elit.";
/// let limit = 20;
/// let result = split_string(input, limit);
///
/// assert_eq!(result, vec![
///     "Lorem ipsum dolor",
///     "sit amet, consectetur",
/// "adipiscing elit."
/// ]);
/// ```
pub fn split_string(input: &str) -> Vec<String> {
    let mut result = Vec::new();
    let mut buffer = String::new();

    for line in input.lines() {
        if buffer.len() + line.len() > 2000 {
            // If the current buffer is not empty, add it to the result.
            if !buffer.is_empty() {
                result.push(buffer);
            }
            // If a single line exceeds the limit, add it on its own.
            if line.len() > 2000 {
                result.push(line.to_string());
                buffer = String::new();
            } else {
                // Otherwise, start a new buffer with this line.
                buffer = line.to_string();
            }
        } else {
            // If adding this line wouldn't exceed the limit, add it to the buffer.
            if !buffer.is_empty() {
                buffer.push('\n');
            }
            buffer.push_str(line);
        }
    }

    // If there's anything left in the buffer, add it to the result.
    if !buffer.is_empty() {
        result.push(buffer);
    }

    result
}