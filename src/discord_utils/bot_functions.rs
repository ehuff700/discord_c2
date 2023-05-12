
use serenity::{
    client::Context,
    model::prelude::ChannelId
};

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
pub async fn send_message(ctx: &Context, channel_id: ChannelId, message: &str, language_format: &str) -> Result<(), String> {

    let output_chunks = message.split('\n');
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