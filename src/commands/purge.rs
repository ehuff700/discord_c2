use serenity::builder::CreateApplicationCommand;
use serenity::model::id::{ChannelId, MessageId};
use serenity::prelude::*;

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    // Create the purge command.
    command
        .name("purge")
        .description("Delete all messages in the channel.")
}

// TODO: Optimize this method, seems to not work well sometimes.
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
