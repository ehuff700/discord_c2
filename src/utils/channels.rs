use crate::errors::DiscordC2Error;
use crate::GUILD_ID;

use serenity::{
    client::Context,
    model::{channel::ChannelType, id::ChannelId},
};

pub async fn create_category_channel(
    ctx: &Context,
    name: String,
) -> Result<ChannelId, DiscordC2Error> {
    let channel_list = GUILD_ID
        .channels(&ctx.http)
        .await
        .map_err(|err| DiscordC2Error::DiscordError(err.to_string()))?;

    for (_, guild_channel) in channel_list.iter() {
        if guild_channel.name == name && guild_channel.kind == ChannelType::Category {
            return Ok(guild_channel.id);
        }
    }

    let category_channel = GUILD_ID
        .create_channel(&ctx.http, |c| c.name(name).kind(ChannelType::Category))
        .await
        .map_err(|err| DiscordC2Error::DiscordError(err.to_string()))?;
    Ok(category_channel.id)
}

pub async fn create_text_channel(
    ctx: &Context,
    name: &str,
    category_id: &ChannelId,
    topic: &str,
) -> Result<ChannelId, DiscordC2Error> {
    let channel_list = GUILD_ID
        .channels(&ctx.http)
        .await
        .map_err(|err| DiscordC2Error::DiscordError(err.to_string()))?;

    for (_, guild_channel) in channel_list.iter() {
        if guild_channel.name == name
            && guild_channel.kind == ChannelType::Text
            && guild_channel.parent_id.unwrap_or(ChannelId(0)) == *category_id
        {
            return Ok(guild_channel.id);
        }
    }

    let text_channel = GUILD_ID
        .create_channel(&ctx.http, |c| {
            c.name(name)
                .kind(ChannelType::Text)
                .category(category_id)
                .topic(topic)
        })
        .await
        .map_err(|err| DiscordC2Error::DiscordError(err.to_string()))?;
    Ok(text_channel.id)
}
